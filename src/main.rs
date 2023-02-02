use crate::config::init_config;

pub mod config;
pub mod history;
pub mod jsonrpc;

fn main() {
    let default_config_path = "config.toml";

    let config = init_config(default_config_path).unwrap();
}
