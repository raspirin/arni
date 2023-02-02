use anyhow::Result;
use arni::Context;
use rss::Channel;
use std::fs::File;
use std::io::BufReader;

fn main() -> Result<()> {
    // init basic context
    let default_config_path = "config.toml";
    let mut context = Context::new(default_config_path)?;

    // read web rss
    if let Some(uris) = context.config.uri {
        for uri in uris {
            let response = context.client.get(&uri).send()?;
            let content = response.bytes()?;
            let channel = Channel::read_from(&content[..])?;
            context.channels.push(channel);
        }
    }
    // read on disk rss
    if let Some(files) = context.config.file {
        for path in files {
            let file = File::open(path)?;
            let channel = Channel::read_from(BufReader::new(file))?;
            context.channels.push(channel);
        }
    }

    for channel in context.channels {
        for item in channel.items {
            println!("{}", item.title().unwrap());
        }
    }

    Ok(())
}
