use anyhow::Result;
use arni::Context;

fn main() -> Result<()> {
    // init basic context
    let default_config_path = "config.toml";
    let mut context = Context::new(default_config_path)?;
    context.prepare_channels()?;

    for channel in context.channels {
        for item in channel.items {
            println!("{}", item.title().unwrap());
        }
    }

    Ok(())
}
