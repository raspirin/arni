use anyhow::Result;
use arni::error::Error;
use arni::history::History;
use arni::jsonrpc::JsonRPCBuilder;
use arni::persist::Persist;
use arni::{get_downloads, init_client, init_config};
use serde_json::Value;

fn main() -> Result<()> {
    // init basic context
    let default_config_path = "config.toml";
    let default_history_path = "history.toml";
    let default_user_agent_name = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"));
    let config = init_config(default_config_path);
    let history = History::load(default_history_path)?;
    let client = init_client(default_user_agent_name);

    // basic loop
    let config = config.reload(default_config_path)?;
    let mut history = history.reload(default_history_path)?;

    let _to_download = get_downloads(&client, &config, &mut history)?;

    let addr = &config.jsonrpc_address;
    let json_rpc = JsonRPCBuilder::new(default_user_agent_name)
        .aria2_get_version(None)
        .build()?;
    let response = json_rpc.send(&client, addr)?.unwrap_response()?;
    println!("{}", response.get("version").unwrap());

    config.write_to_disk(default_config_path)?;
    history.write_to_disk(default_history_path)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
}
