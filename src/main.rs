use anyhow::{Context, Result};
use arni::config::Config;
use arni::history::History;
use arni::jsonrpc::JsonRPCBuilder;
use arni::persist::Persist;
use arni::{get_downloads, init_client, DownloadStatus, Episode, send_to_aria2};
use reqwest::blocking::Client;
use std::time::Duration;

static CONFIG_PATH: &str = "config.toml";
static HISTORY_PATH: &str = "history.toml";
static UA: &str = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"));

fn main() -> Result<()> {
    // init basic context
    let mut config = Config::load(CONFIG_PATH).context("Fail to load/create config file.")?;
    // TODO: replace the history instance with sqlite instance
    let mut history = History::load(HISTORY_PATH).context("Fail to load/create history file.")?;
    let client = init_client(UA);
    let mut download_list: Vec<Episode> = vec![];

    let mut first_loop = true;
    loop {
        if first_loop {
            first_loop = false;
        } else {
            std::thread::sleep(Duration::from_secs(3600));
        }

        // TODO: improve error handling, filter out what we can do when something fails
        // everything in this loop should never cause panicking

        // basic loop
        // TODO: reload this two file when needed
        if config.reload(CONFIG_PATH).is_err() {
            continue;
        }
        if history.reload(HISTORY_PATH).is_err() {
            continue;
        }

        // TODO: simplify this function
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

#[cfg(test)]
mod tests {
    use super::*;
}
