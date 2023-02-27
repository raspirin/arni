use std::string::ToString;
use anyhow::{Context, Result};
use arni::config::Config;
use arni::history::History;
use arni::persist::Persist;
use arni::{init_client, send_to_aria2, DownloadStatus, Episode, merge_download_list, sync_download_status, dry_send_to_aria2};
use std::time::Duration;

static CONFIG_PATH: &str = "config.toml";
static HISTORY_PATH: &str = "history.toml";
static UA: &str = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"));

fn main() -> Result<()> {
    let dry_run: String = std::env::var("ARNI_DRY_RUN").unwrap_or("false".to_string());
    // init basic context
    let mut config = Config::load(CONFIG_PATH).context("Fail to load/create config file.")?;
    // TODO: replace the history instance with sqlite instance
    let mut history = History::load(HISTORY_PATH).context("Fail to load/create history file.")?;
    let client = init_client(UA);
    let mut download_list: Vec<Episode> = vec![];

    let mut first_loop = true;
    loop {
        // basic loop
        // TODO: improve error handling, filter out what we can do when something fails
        // everything in this loop should never cause panicking

        if first_loop {
            first_loop = false;
        } else {
            std::thread::sleep(Duration::from_secs(3600));
        }

        // TODO: reload this two file when needed
        if config.reload(CONFIG_PATH).is_err() {
            continue;
        }
        if history.reload(HISTORY_PATH).is_err() {
            continue;
        }

        // TODO: simplify this function
        if merge_download_list(&mut config, &mut history, &client, &mut download_list).is_ok() {
            if dry_run == "false" {
                let _ = send_to_aria2(UA, &client, &config, &mut download_list);
            } else {
                let _ = dry_send_to_aria2(UA, &client, &config, &download_list);
            }
        }

        // just ignore when this function returns an error
        // we can sync next wakeup
        if dry_run == "false" {
            let _ = sync_download_status(UA, &client, &config, &mut history, &mut download_list);
        }

        download_list.retain(|episode| {
            !matches!(
                &episode.download_status,
                DownloadStatus::Done | DownloadStatus::Error
            )
        });

        // TODO: impl sync function of these two in case of failing to write to disk
        config.write_to_disk(CONFIG_PATH)?;
        history.write_to_disk(HISTORY_PATH)?;
    }
}


#[cfg(test)]
mod tests {
    use super::*;
}
