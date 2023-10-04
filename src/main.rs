use anyhow::{Context, Result};
use arni::history::History;
use arni::persist::Persist;
use arni::{
    dry_send_to_aria2, init_client, merge_download_list, send_to_aria2, sync_download_status,
    DownloadStatus, Episode,
};
use assets_manager::AssetCache;
use std::string::ToString;
use std::time::Duration;

use arni::novel_config::NovelConfig;

static HISTORY_PATH: &str = "history.toml";
static UA: &str = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"));

fn main() -> Result<()> {
    let dry_run: String = std::env::var("ARNI_DRY_RUN").unwrap_or_else(|_| "false".to_string());

    // init basic context
    // TODO: replace the history instance with sqlite instance
    let persist_folder_path = "persist";
    let novel_config_path = "novel_config";
    let persist_cache = AssetCache::new(persist_folder_path)?;
    let config_handle = persist_cache.load::<NovelConfig>(novel_config_path)?;

    let mut history = History::load(HISTORY_PATH).context("Fail to load/create history file.")?;
    let client = init_client(UA);
    let mut download_list: Vec<Episode> = vec![];

    let mut first_loop = true;
    loop {
        // basic loop
        // TODO: improve error handling, filter out what we can do when something fails
        // everything in this loop should never cause panicking

        persist_cache.hot_reload();
        let novel_config = config_handle.read();

        if first_loop {
            first_loop = false;
        } else {
            std::thread::sleep(Duration::from_secs(3600));
        }

        // TODO: reload this two file when needed
        if history.reload(HISTORY_PATH).is_err() {
            continue;
        }

        // TODO: simplify this function
        if merge_download_list(&novel_config, &mut history, &client, &mut download_list).is_ok() {
            if dry_run == "false" {
                let _ = send_to_aria2(UA, &client, &novel_config, &mut download_list);
            } else {
                let _ = dry_send_to_aria2(UA, &client, &novel_config, &download_list);
            }
        }

        // just ignore when this function returns an error
        // we can sync next wakeup
        if dry_run == "false" {
            let _ = sync_download_status(UA, &client, &novel_config, &mut history, &mut download_list);
        }

        download_list.retain(|episode| {
            !matches!(
                &episode.download_status,
                DownloadStatus::Done | DownloadStatus::Error
            )
        });

        // TODO: impl sync function of these two in case of failing to write to disk
        history.write_to_disk(HISTORY_PATH)?;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_main_with_novel_config() -> Result<()> {
        let persist_path = "persist";
        let config_path = "novel_config";
        let persist_cache = AssetCache::new(persist_path)?;
        let config_handle = persist_cache.load::<NovelConfig>(config_path)?;

        let config = config_handle.read();

        assert_eq!(config.jsonrpc_address, "127.0.0.1:16800");

        Ok(())
    }
}
