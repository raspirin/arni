use crate::config::Config;
use crate::error::Error;
use crate::history::History;
use crate::persist::Persist;
use anyhow::Result;
use reqwest::blocking::Client;
use rss::Channel;
use std::fs::File;
use std::io::BufReader;

pub mod config;
pub mod error;
pub mod history;
pub mod jsonrpc;
pub mod persist;

pub enum DownloadStatus {
    Waiting,
    Sent,
    Done,
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
}

pub fn init_config(config_path: &str) -> Config {
    match Config::load(config_path) {
        Ok(config) => config,
        Err(e) => panic!("Fail to load/create config. {e}"),
    }
}

pub fn init_client(user_agent: &str) -> Client {
    match Client::builder().user_agent(user_agent).build() {
        Ok(c) => c,
        Err(e) => panic!("Fail to create a web client. {e}"),
    }
}

fn get_channels(config: &Config, client: &Client) -> Result<Vec<Channel>> {
    let mut ret: Vec<Channel> = vec![];

    if let Some(uris) = &config.uri {
        for uri in uris {
            let channel = read_web_rss(uri, client)?;
            ret.push(channel);
        }
    }

    if let Some(files) = &config.file {
        for file in files {
            let channel = read_on_disk_rss(file)?;
            ret.push(channel);
        }
    }

    Ok(ret)
}

fn read_web_rss(uri: &str, client: &Client) -> Result<Channel> {
    let response = client.get(uri).send()?;
    let content = response.bytes()?;

    Ok(Channel::read_from(&content[..])?)
}

fn read_on_disk_rss(path: &str) -> Result<Channel> {
    let file = File::open(path)?;

    Ok(Channel::read_from(BufReader::new(file))?)
}

fn get_episodes(channels: &Vec<Channel>) -> Result<Vec<Episode>> {
    let mut ret: Vec<Episode> = vec![];
    for channel in channels {
        for item in channel.items() {
            push_episode(&mut ret, item)?;
        }
    }

    Ok(ret)
}

fn push_episode(vec: &mut Vec<Episode>, item: &rss::Item) -> Result<()> {
    let torrent_link = match item.enclosure() {
        Some(enclosure) => enclosure.url().to_string(),
        None => return Err(anyhow::Error::from(Error::BadTorrentLink)),
    };
    let guid = match item.guid() {
        Some(guid) => guid.value().to_string(),
        // if there is None, the function will return before this unwrap()
        None => item.enclosure().unwrap().url().to_string(),
    };
    let title = item.title().map(|title| title.to_string());

    let episode = Episode::new(guid, title, torrent_link);
    vec.push(episode);

    Ok(())
}

pub fn get_downloads(
    client: &Client,
    config: &Config,
    history: &mut History,
) -> Result<Vec<Episode>> {
    let channels = get_channels(config, client)?;
    let episodes = get_episodes(&channels)?;
    let mut to_download: Vec<Episode> = vec![];
    for episode in episodes.into_iter() {
        if !history.query_downloaded(&episode.guid) {
            to_download.push(episode)
        }
    }
    Ok(to_download)
}

#[cfg(test)]
mod tests {
    use super::*;
}
