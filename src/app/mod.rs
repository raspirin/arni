use std::{fs::File, io::BufReader};

use anyhow::{Context, Result};
use log::{error, info, warn};
use rss::Channel;

use crate::{
    client::{Client, UA},
    data::{config::Config, history::History, SyncFile},
    data::episode::Episode,
    error::Error,
    jsonrpc::JsonRPCBuilder,
};

pub struct App<'a> {
    pub config: &'a mut Config<'a>,
    pub history: &'a mut History<'a>,
    pub client: Client,
    download_list: Vec<Episode>,
    ua: UA,
}

impl<'a> App<'a> {
    pub fn new(config: &'a mut Config<'a>, history: &'a mut History<'a>) -> Result<Self> {
        Self::with_ua(config, history)
    }

    pub fn with_ua(config: &'a mut Config<'a>, history: &'a mut History<'a>) -> Result<Self> {
        info!("Creating in-app client...");
        let client = Client::new().map_err(|e| {
            warn!("Fail to create in-app client");
            e
        })?;

        let ret = Self {
            config,
            history,
            client,
            download_list: vec![],
            ua: UA::default(),
        };

        Ok(ret)
    }

    pub fn run(&mut self, dry_run: bool) -> Result<()> {
        if self.check_aria2_connection() && !dry_run {
            info!("Can't connect to aria2.");
            info!("waiting for next loop.");
            return Err(Error::Aria2ConnectionError.into());
        }

        // reload
        info!("P1 syncing config");
        self.config
            .sync()
            .with_context(|| "Can't sync config (p1)")
            .map_err(|e| {
                warn!("P1 config sync failed: {}", e);
                e
            })?;
        info!("P1 syncing history");
        self.history
            .sync()
            .with_context(|| "Can't sync history (p1)")
            .map_err(|e| {
                warn!("P1 history sync failed: {}", e);
                e
            })?;

        // get episodes from rss
        info!("Getting episodes from rss...");
        info!("Getting rss channels...");
        let channels = self.get_rss_channels().map_err(|e| {
            warn!("Fail to getting rss channels: {e}");
            e
        })?;
        let mut episodes: Vec<Episode> = vec![];
        info!("Collecting episodes...");
        for channel in channels {
            for item in channel.items {
                episodes.push(Episode::try_from(item).map_err(|e| {
                    warn!("Can't convert Item into Episode: {e}");
                    e
                })?)
            }
        }
        let episodes = episodes
            .into_iter()
            .filter(|epi| self.history.query(&epi.guid))
            // merge episodes into download_list
            // download_list contains episode that we've sent to aria2
            .filter(|epi| self.download_list.contains(epi));

        self.download_list
            .append(&mut episodes.collect::<Vec<Episode>>());

        // send episode to aria2
        info!("Sending episodes to aria2");
        for epi in self
            .download_list
            .iter_mut()
            // only takes out what we need to send
            .filter(|epi| epi.is_waiting())
        {
            let jsonrpc = JsonRPCBuilder::new(self.ua.as_str())
                .aria2_add_uri(None, &epi.torrent_link)
                .build()
                .map_err(|e| {
                    warn!("Fail to build JsonRPC: {e}");
                    e
                })?;
            if !dry_run {
                let response = self
                    .client
                    .send(self.config.aria2_address(), jsonrpc)
                    .map_err(|e| {
                        warn!("Fail to get JsonRPC's response: {e}");
                        e
                    })?;
                // TODO: will this panic?
                let gid = response.unwrap_response()?.get("gid").unwrap().to_string();
                epi.gid = Some(gid);
                epi.set_sent();
            } else {
                let response = self.client.dry_send(self.config.aria2_address(), jsonrpc)?;
                println!("dry run: {}", response);
            }
        }

        // sync download status
        info!("Syncing download status");
        for epi in self.download_list.iter_mut().filter(|epi| epi.is_sent()) {
            let jsonrpc = JsonRPCBuilder::new(self.ua.as_str())
                .aria2_tell_status(None, &epi.gid()?)
                .build()
                .map_err(|e| {
                    warn!("Fail to build JsonRPC: {e}");
                    e
                })?;
            if !dry_run {
                let response = self
                    .client
                    .send(self.config.aria2_address(), jsonrpc)
                    .map_err(|e| {
                        warn!("Fail to get JsonRPC's response: {e}");
                        e
                    })?;
                let status = response
                    .unwrap_response()?
                    .get("status")
                    .unwrap()
                    .to_string();
                epi.set_download_status(&status)?;
            } else {
                let response = self.client.dry_send(self.config.aria2_address(), jsonrpc)?;
                println!("dry run: {}", response)
            }
        }

        // update history
        info!("Updating history...");
        for epi in self.download_list.iter().filter(|epi| epi.is_done()) {
            self.history.push(&epi.guid);
        }

        // remove items in download_list
        self.download_list.retain(|epi| !epi.is_done());

        // write back
        info!("P2 syncing config...");
        self.config.sync().map_err(|e| {
            warn!("P2 config sync failed: {e}");
            e
        })?;
        info!("P2 syncing history...");
        self.history.sync().map_err(|e| {
            warn!("P2 history sync failed: {e}");
            e
        })?;

        Ok(())
    }

    fn get_rss_channels(&mut self) -> Result<Vec<Channel>> {
        let mut ret: Vec<Channel> = vec![];

        // read on disk rss channel
        if let Some(files) = &self.config.file() {
            for file in files.iter() {
                let file = File::open(file)?;
                let from_file = Channel::read_from(BufReader::new(file))?;

                ret.push(from_file);
            }
        }

        // read web rss channel
        if let Some(urls) = &self.config.url() {
            for url in urls.iter() {
                let response = self.client.inner().post(url).send()?;
                let content = response.bytes()?;
                let from_web = Channel::read_from(&content[..])?;

                ret.push(from_web);
            }
        }

        Ok(ret)
    }

    fn check_aria2_connection(&mut self) -> bool {
        info!("Checking connectin with aria2");
        let jsonrpc = JsonRPCBuilder::new(self.ua.as_str())
            .aria2_get_version(None)
            .build();
        let jsonrpc = match jsonrpc {
            Ok(jsonrpc) => jsonrpc,
            Err(_e) => {
                error!("Can't build jsonrpc");
                return false;
            }
        };
        let response = self
            .client
            .send(self.config.aria2_address(), jsonrpc)
            .map_err(|e| {
                error!("Can't get response from aria2.");
                e
            });
        let response = match response {
            Ok(r) => r,
            Err(_e) => return false,
        };
        let response = response.unwrap_response().unwrap();
        let version = response.get("version").unwrap();
        info!("Connection with aria2: {version}");
        true
    }
}
