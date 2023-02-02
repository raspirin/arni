use std::fs::File;
use std::io::BufReader;
use crate::config::Config;
use anyhow::{Result};
use reqwest::blocking::Client;
use rss::Channel;

pub mod config;
pub mod history;
pub mod jsonrpc;

pub struct Context {
    pub config: Config,
    pub client: Client,
    pub channels: Vec<Channel>,
}

impl Context {
    pub fn new(config_path: &str) -> Result<Self> {
        let config = Self::new_config(config_path);
        let client = Self::new_client()?;

        Ok(Self {
            config,
            client,
            channels: vec![],
        })
    }

    fn new_config(path: &str) -> Config {
        match config::load_config(path) {
            Ok(config) => config,
            Err(e) => panic!("Fail to load/create config. {e}"),
        }
    }

    fn new_client() -> Result<Client> {
        let user_agent = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"));
        let client = match Client::builder().user_agent(user_agent).build() {
            Ok(c) => c,
            Err(e) => panic!("Fail to create a web client. {e}")
        };
        Ok(client)
    }

    pub fn prepare_channels(&mut self) -> Result<()> {
        if let Some(uris) = &self.config.uri {
            for uri in uris {
                let channel = self.read_web_rss(uri)?;
                self.channels.push(channel);
            }
        }

        if let Some(files) = &self.config.file {
            for file in files {
                let channel = Self::read_on_disk_rss(file)?;
                self.channels.push(channel);
            }
        }

        Ok(())
    }

    fn read_web_rss(&self, uri: &str) -> Result<Channel> {
        let response = self.client.get(uri).send()?;
        let content = response.bytes()?;
        Ok(Channel::read_from(&content[..])?)
    }

    fn read_on_disk_rss(path: &str) -> Result<Channel> {
        let file = File::open(path)?;
        Ok(Channel::read_from(BufReader::new(file))?)
    }
}
