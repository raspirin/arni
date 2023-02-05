use anyhow::Result;
use arni::history::History;
use arni::persist::Persist;
use arni::{error::Error, get_channels, get_episodes, init_client, init_config, Episode};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::File;
use std::io::{Read, Write};

fn main() -> Result<()> {
    // init basic context
    let default_config_path = "config.toml";
    let default_history_path = "history.toml";
    let config = init_config(default_config_path);
    let history = History::load(default_history_path)?;
    let client = init_client();

    // basic loop
    let config = config.reload(default_config_path)?;
    let mut history = history.reload(default_history_path)?;
    let channels = get_channels(&config, &client)?;
    let episodes = get_episodes(&channels)?;
    let mut to_download: Vec<Episode> = vec![];
    for episode in episodes.into_iter() {
        if !history.query_downloaded(&episode.guid) {
            to_download.push(episode)
        }
    }
    for episode in to_download.iter() {
        println!("{:?}", history.query_downloaded(&episode.guid))
    }
    config.write_to_disk(default_config_path)?;
    history.write_to_disk(default_history_path)?;

    Ok(())
}
