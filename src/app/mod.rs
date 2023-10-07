use std::{fs::File, io::BufReader};

use anyhow::Result;
use rss::Channel;

use crate::{
    client::Client,
    config::{Config, History, SyncFile},
    error::Error,
    jsonrpc::JsonRPCBuilder,
};

pub struct UA {
    inner: String,
}

impl UA {
    pub fn new(ua: &str) -> Self {
        Self {
            inner: ua.to_string(),
        }
    }

    pub fn as_str(&self) -> &str {
        &self.inner
    }

    pub fn into_string(self) -> String {
        self.inner
    }
}

impl Default for UA {
    fn default() -> Self {
        Self {
            inner: concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION")).to_string(),
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum DownloadStatus {
    /// Waiting for sending to aria2
    Waiting,
    /// Sent to aria2
    Sent,
    /// Finished downloading
    Done,
    /// Something went wrong on the aria2 side
    Error,
}

pub struct Episode {
    pub guid: String,
    pub title: Option<String>,
    pub torrent_link: String,
    pub gid: Option<String>,
    pub download_status: DownloadStatus,
}

impl Episode {
    pub fn new(guid: String, title: Option<String>, torrent_link: String) -> Self {
        Self {
            guid,
            title,
            torrent_link,
            gid: None,
            download_status: DownloadStatus::Waiting,
        }
    }

    pub fn is_waiting(&self) -> bool {
        self.download_status == DownloadStatus::Waiting
    }

    pub fn is_sent(&self) -> bool {
        self.download_status == DownloadStatus::Sent
    }

    pub fn is_done(&self) -> bool {
        self.download_status == DownloadStatus::Done
    }

    pub fn set_download_status(&mut self, status: &str) -> Result<(), Error> {
        self.download_status = match status {
            "active" | "waiting" | "paused" => DownloadStatus::Sent,
            "error" => DownloadStatus::Error,
            "complete" | "removed" => DownloadStatus::Done,
            _ => return Err(Error::ImpossibleEpisodeState),
        };

        Ok(())
    }

    pub fn set_sent(&mut self) {
        self.download_status = DownloadStatus::Sent;
    }

    pub fn gid(&self) -> Result<String, Error> {
        match &self.gid {
            Some(gid) => Ok(gid.to_string()),
            None => Err(Error::ImpossibleEpisodeState),
        }
    }
}

impl PartialEq for Episode {
    fn eq(&self, other: &Self) -> bool {
        self.guid == other.guid
    }
}

impl TryFrom<rss::Item> for Episode {
    type Error = Error;

    fn try_from(value: rss::Item) -> std::result::Result<Self, Self::Error> {
        let torrent_link = match value.enclosure() {
            Some(enclosure) => enclosure.url().to_string(),
            None => return Err(Error::BadTorrentLink),
        };
        let guid = match value.guid() {
            Some(guid) => guid.value().to_string(),
            // if there is None, the function will return before this unwrap()
            None => value.enclosure().unwrap().url().to_string(),
        };
        let title = value.title().map(|title| title.to_string());
        Ok(Self::new(guid, title, torrent_link))
    }
}

pub struct App<'a> {
    pub config: &'a mut Config<'a>,
    pub history: &'a mut History<'a>,
    pub client: Client,
    ua: UA,
    download_list: Vec<Episode>,
}

impl<'a> App<'a> {
    pub fn new(config: &'a mut Config<'a>, history: &'a mut History<'a>) -> Result<Self> {
        Self::with_ua(config, history, UA::default())
    }

    pub fn with_ua(
        config: &'a mut Config<'a>,
        history: &'a mut History<'a>,
        ua: UA,
    ) -> Result<Self> {
        let client = Client::with_ua(ua.as_str())?;
        Ok(Self {
            config,
            history,
            client,
            ua,
            download_list: vec![],
        })
    }

    pub fn run(&mut self, dry_run: bool) -> Result<()> {
        // reload
        self.config.sync()?;
        self.history.sync()?;

        // get episodes from rss
        let channels = self.get_rss_channels()?;
        let mut episodes: Vec<Episode> = vec![];
        for channel in channels {
            for item in channel.items {
                episodes.push(Episode::try_from(item)?)
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
        for epi in self
            .download_list
            .iter_mut()
            // only takes out what we need to send
            .filter(|epi| epi.is_waiting())
        {
            let jsonrpc = JsonRPCBuilder::new(&self.ua.inner)
                .aria2_add_uri(None, &epi.torrent_link)
                .build()?;
            if !dry_run {
                let response = self.client.send(self.config.aria2_address(), jsonrpc)?;
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
        for epi in self.download_list.iter_mut().filter(|epi| epi.is_sent()) {
            let jsonrpc = JsonRPCBuilder::new(&self.ua.inner)
                .aria2_tell_status(None, &epi.gid()?)
                .build()?;
            if !dry_run {
                let response = self.client.send(self.config.aria2_address(), jsonrpc)?;
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
        for epi in self.download_list.iter().filter(|epi| epi.is_done()) {
            self.history.push(&epi.guid);
        }

        // remove items in download_list
        self.download_list.retain(|epi| !epi.is_done());

        // write back
        self.config.sync()?;
        self.history.sync()?;

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
}
