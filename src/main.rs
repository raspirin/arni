use anyhow::Result;
use arni::history::History;
use arni::persist::Persist;
use arni::{init_client, init_config, get_downloads};

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

    let _to_download = get_downloads(&client, &config, &mut history)?;

    config.write_to_disk(default_config_path)?;
    history.write_to_disk(default_history_path)?;

    Ok(())
}


