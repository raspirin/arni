use std::time::Duration;

use anyhow::Result;
use arni::{
    app::App,
    config::{Config, History},
};
use clap::Parser;

#[derive(Debug, Parser)]
#[command(author, version, about, long_about=None)]
struct Cli {
    #[arg(short, long)]
    watch: bool,

    #[arg(long)]
    dry_run: bool,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    let config = "config.toml";
    let mut config = Config::new(config)?;

    let history = "history.toml";
    let mut history = History::new(history)?;

    let mut app = App::new(&mut config, &mut history)?;

    if cli.watch {
        loop {
            let _ = app.run(cli.dry_run);
            std::thread::sleep(Duration::from_secs(3600));
        }
    } else {
        app.run(cli.dry_run)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
}
