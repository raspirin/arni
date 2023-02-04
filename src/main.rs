use anyhow::Result;
use arni::{error::Error, get_channels, get_episodes, init_client, init_config};

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
