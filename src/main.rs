use anyhow::Result;
use arni::config::Config;
use arni::history::History;
use arni::jsonrpc::JsonRPCBuilder;
use arni::persist::Persist;
use arni::{get_downloads, init_client, init_config, Episode, DownloadStatus};
use reqwest::blocking::Client;

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

    let mut download_list = get_downloads(&client, &config, &mut history)?;

    send_to_aria2(
        default_user_agent_name,
        &client,
        &config,
        &mut download_list,
    )?;

    let addr = &config.jsonrpc_address;
    for episode in download_list.iter_mut() {
        let response = JsonRPCBuilder::new(default_user_agent_name)
            .aria2_tell_status(None, &episode.gid.clone().unwrap())
            .build()?
            .send(&client, addr)?
            .unwrap_response()?;
        let status = response.get("status").unwrap().as_str();
        episode.download_status =  match status {
            "active" | "waiting" | "paused" => DownloadStatus::Sent,
            "error" => DownloadStatus::Error,
            "complete" | "removed" => DownloadStatus::Done,
            _ => panic!("impossible download status")
        };

        history.get_metadata_mut(&episode.guid).is_downloaded = matches!(&episode.download_status, DownloadStatus::Done | DownloadStatus::Error);
    }

    config.write_to_disk(default_config_path)?;
    history.write_to_disk(default_history_path)?;

    Ok(())
}

fn send_to_aria2(
    default_user_agent_name: &str,
    client: &Client,
    config: &Config,
    download_list: &mut Vec<Episode>,
) -> Result<()> {
    let addr = &config.jsonrpc_address;
    for mut episode in download_list {
        let response = JsonRPCBuilder::new(default_user_agent_name)
            .aria2_add_uri(None, &episode.torrent_link)
            .build()?
            .send(client, addr)?
            .unwrap_response()?;
        let gid = response.get("gid").unwrap().clone();
        episode.gid = Some(gid);
        episode.download_status = DownloadStatus::Sent;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
}
