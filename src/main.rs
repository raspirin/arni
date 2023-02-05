use anyhow::Result;
use reqwest::blocking::Client;
use arni::error::Error;
use arni::history::History;
use arni::jsonrpc::JsonRPCBuilder;
use arni::persist::Persist;
use arni::{Episode, get_downloads, init_client, init_config};
use serde_json::Value;
use arni::config::Config;

fn main() -> Result<()> {
    // init basic context
    let default_config_path = "config.toml";
    let default_history_path = "history.toml";
    let default_user_agent_name = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"));
    let config = init_config(default_config_path);
    let history = History::load(default_history_path)?;
    let client = init_client(default_user_agent_name);

    // basic loop
    let config = config.reload(default_config_path)?;
    let mut history = history.reload(default_history_path)?;

    let to_download = get_downloads(&client, &config, &mut history)?;

    send_to_aria2(default_user_agent_name, &client, &config, &mut history, &to_download)?;

    config.write_to_disk(default_config_path)?;
    history.write_to_disk(default_history_path)?;

    Ok(())
}

fn send_to_aria2(default_user_agent_name: &str, client: &Client, config: &Config, history: &mut History, to_download: &Vec<Episode>) -> Result<()> {
    let addr = &config.jsonrpc_address;
    for episode in to_download {
        let response = JsonRPCBuilder::new(default_user_agent_name)
            .aria2_add_uri(None, &episode.torrent_link, None, None)
            .build()?
            .send(client, addr)?
            .unwrap_response()?;
        let gid: u64 = response.get("gid").unwrap().parse()?;
        let metadata = history.get_metadata_mut(&episode.guid);
        metadata.gid = Some(gid);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
}
