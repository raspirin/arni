use anyhow::Result;
use arni::config::Config;
use arni::history::History;
use arni::jsonrpc::JsonRPCBuilder;
use arni::persist::Persist;
use arni::{get_downloads, init_client, init_config, DownloadStatus, Episode};
use reqwest::blocking::Client;
use std::time;

static CONFIG_PATH: &str = "config.toml";
static HISTORY_PATH: &str = "history.toml";
static UA: &str = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"));

fn main() -> Result<()> {
    // init basic context
    let mut config = init_config(CONFIG_PATH);
    let mut history = History::load(HISTORY_PATH)?;
    let client = init_client(UA);
    let mut download_list: Vec<Episode> = vec![];

    loop {
        // basic loop
        let _ = config.reload(CONFIG_PATH);
        let _ = history.reload(HISTORY_PATH);

        let _ = merge_download_list(&mut config, &mut history, &client, &mut download_list);

        let _ = send_to_aria2(
            UA,
            &client,
            &config,
            &mut download_list,
        );

        let _ = sync_download_status(
            UA,
            &client,
            &config,
            &mut history,
            &mut download_list,
        );

        download_list.retain(|episode| {
            !matches!(
                &episode.download_status,
                DownloadStatus::Done | DownloadStatus::Error
            )
        });

        config.write_to_disk(CONFIG_PATH)?;
        history.write_to_disk(HISTORY_PATH)?;

        let duration = time::Duration::from_secs(3600);
        std::thread::sleep(duration);
    }
}

fn merge_download_list(
    config: &mut Config,
    history: &mut History,
    client: &Client,
    download_list: &mut Vec<Episode>,
) -> Result<()> {
    let to_merge = get_downloads(client, config, history)?;
    for episode in to_merge.into_iter() {
        let mut unique = true;
        for download in download_list.iter() {
            if download.guid == episode.guid {
                unique = false;
            }
        }
        if unique {
            download_list.push(episode);
        }
    }

    Ok(())
}

fn sync_download_status(
    default_user_agent_name: &str,
    client: &Client,
    config: &Config,
    history: &mut History,
    download_list: &mut [Episode],
) -> Result<()> {
    let addr = &config.jsonrpc_address;
    for episode in download_list.iter_mut() {
        let response = JsonRPCBuilder::new(default_user_agent_name)
            .aria2_tell_status(None, &episode.gid.clone().unwrap())
            .build()?
            .send(client, addr)?
            .unwrap_response()?;
        let status = response.get("status").unwrap().as_str();
        episode.download_status = match status {
            "active" | "waiting" | "paused" => DownloadStatus::Sent,
            "error" => DownloadStatus::Error,
            "complete" | "removed" => DownloadStatus::Done,
            _ => panic!("impossible download status"),
        };

        history.get_metadata_mut(&episode.guid).is_downloaded = matches!(
            &episode.download_status,
            DownloadStatus::Done | DownloadStatus::Error
        );
    }

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
        if matches!(&episode.download_status, DownloadStatus::Waiting) {
            let response = JsonRPCBuilder::new(default_user_agent_name)
                .aria2_add_uri(None, &episode.torrent_link)
                .build()?
                .send(client, addr)?
                .unwrap_response()?;
            let gid = response.get("gid").unwrap().clone();
            episode.gid = Some(gid);
            episode.download_status = DownloadStatus::Sent;
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
}
