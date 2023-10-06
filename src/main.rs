use anyhow::Result;
use arni::{
    app::App,
    config::{Config, History},
};

fn main() -> Result<()> {
    let dry_run: bool = std::env::var("ARNI_DRY_RUN").unwrap_or_else(|_| "false".to_string()).trim().parse()?;

    let config = "config.toml";
    let mut config = Config::new(config)?;

    let history = "history.toml";
    let mut history = History::new(history)?;

    let mut app = App::new(&mut config, &mut history)?;

    loop {
        let _ = app.run(dry_run);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
}
