use std::{time::Duration};

use anyhow::{Context, Result};
use arni::{
    app::App,
    data::{config::Config, history::History},
};
use clap::Parser;
use log::{error, info};

#[derive(Debug, Parser)]
#[command(author, version, about, long_about=None)]
struct Cli {
    /// Run continously
    #[arg(short, long)]
    watch: bool,

    /// Directory of config and history
    #[arg(short = 'd', long = "working_directory", value_name = "PATH")]
    working_dir: Option<String>,

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
    let config = if let Some(dir) = &cli.working_dir {
        format!("{dir}/{config}")
    } else {
        config.to_string()
    };
    let mut config = Config::new(&config)
        .with_context(|| "Init config failed.")
        .map_err(|e| {
            error!("Can't init config: {e}");
            e
        })?;

    info!("Init history...");
    let history = "history.toml";
    let history = if let Some(dir) = &cli.working_dir {
        format!("{dir}/{history}")
    } else {
        history.to_string()
    };
    let mut history = History::new(&history)
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
    
}
