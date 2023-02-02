use std::fs::File;
use std::io::{BufReader, Read};
use std::str::FromStr;
use crate::config::{Config, init_config};
use reqwest::blocking as request;
use anyhow::Result;
use rss::Channel;

pub mod config;
pub mod history;
pub mod jsonrpc;

fn main() -> Result<()>{
    // init config
    let default_config_path = "config.toml";
    let config = match init_config(default_config_path) {
        Ok(config) => config,
        Err(_) => panic!("Fail to load/create config."),
    };

    // channels
    let mut channels: Vec<Channel> = vec![];
    // read web rss
    let user_agent = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"));
    let rss_client = request::Client::builder().user_agent(user_agent).build()?;
    if let Some(uris) = config.uri {
        for uri in uris {
            let response = rss_client.get(&uri).send()?;
            let content = response.bytes()?;
            let channel = Channel::read_from(&content[..])?;
            channels.push(channel);
        }
    }
    // read on disk rss
    if let Some(files) = config.file {
        for path in files {
            let mut file = File::open(path)?;
            let channel = Channel::read_from(BufReader::new(file))?;
            channels.push(channel);
        }
    }

    for channel in channels {
        for item in channel.items {
            println!("{}", item.title().unwrap());
        }
    }

    Ok(())
}