use std::collections::HashMap;
use std::fs::File;
use std::io::{Read, Write};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use arni::{error::Error, get_channels, get_episodes, init_client, init_config};
use arni::persist::Persist;

fn main() -> Result<()> {
    // init basic context
    let default_config_path = "config.toml";
    let config = init_config(default_config_path);
    let client = init_client();

    // basic loop
    let channels = get_channels(&config, &client)?;
    let episodes = get_episodes(&channels)?;

    Ok(())
}


#[derive(Serialize, Deserialize)]
struct History {
    inner: Option<HashMap<String, (bool, u64)>>,
}

impl History {
    fn new() -> Self {
        Self {
            inner: Some(HashMap::new()),
        }
    }


}

impl Persist for History {
    fn new_empty() -> Self {
        History::new()
    }

    fn from_str(s: &str) -> Result<Self> {
        let ret: History = toml::from_str(s)?;
        Ok(ret)
    }

    fn to_string(&self) -> Result<String> {
        let ret = toml::to_string(&self)?;
        Ok(ret)
    }
}