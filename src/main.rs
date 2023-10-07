use std::time::Duration;

use anyhow::{Context, Result};
use arni::{
    app::App,
    config::{Config, History},
};
use clap::Parser;
use log::{error, info};

#[derive(Debug, Parser)]
#[command(author, version, about, long_about=None)]
struct Cli {
    #[arg(short, long)]
    watch: bool,

    #[arg(long)]
    dry_run: bool,
}

fn main() -> Result<()> {
    pretty_env_logger::init();
    info!("Hello, welcome to arni.");
    info!("Parsing cli args...");
    let cli = Cli::parse();

    info!("Init config...");
    let config = "config.toml";
    let mut config = Config::new(config)
        .with_context(|| "Init config failed.")
        .map_err(|e| {
            error!("Can't init config: {e}");
            e
        })?;

    info!("Init history...");
    let history = "history.toml";
    let mut history = History::new(history)
        .with_context(|| "Init history failed.")
        .map_err(|e| {
            error!("Can't init history: {e}");
            e
        })?;

    info!("Starting app...");
    let mut app = App::new(&mut config, &mut history)?;

    if cli.watch {
        info!("Entering watch mode.");
        loop {
            let _ = app.run(cli.dry_run);
            std::thread::sleep(Duration::from_secs(3600));
        }
    } else {
        info!("Entering one-shot mode.");
        app.run(cli.dry_run)?;
    }
    info!("Shutting down...");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
}
