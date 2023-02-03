use anyhow::Result;
use arni::Context;

fn main() -> Result<()> {
    // init basic context
    let default_config_path = "config.toml";
    let mut context = Context::new(default_config_path)?;
    context.prepare_channels()?;

    for channel in context.channels.iter() {
        for item in channel.items().iter() {
            println!("{}", item.enclosure().unwrap().url())
        }
    }

    Ok(())
}
