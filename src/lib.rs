use crate::config::Config;
use anyhow::Result;
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
            Err(_) => panic!("Fail to load/create config."),
        }
    }

    fn new_client() -> Result<Client> {
        let user_agent = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"));
        Ok(Client::builder().user_agent(user_agent).build()?)
    }
}
