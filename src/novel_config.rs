use anyhow::Result;
use assets_manager::{loader, Asset, AssetCache};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct NovelConfig {
    pub jsonrpc_address: String,
    pub uri: Option<Vec<String>>,
    pub file: Option<Vec<String>>,
}

impl Asset for NovelConfig {
    const EXTENSION: &'static str = "toml";
    type Loader = loader::TomlLoader;
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::fs::File;
    use std::io::{Read, Write};

    #[test]
    fn test_config() -> Result<()> {
        let s = r#"jsonrpc_address = "127.0.0.1:16800"
uri = ["test_novel_config"]
file = []"#;
        let path = "persist/test_novel_config.toml";
        let mut file = File::create(path)?;
        file.write_all(s.as_bytes())?;

        let config_cache = AssetCache::new("persist").unwrap();
        let config_handle = config_cache
            .load::<NovelConfig>("test_novel_config")
            .unwrap();
        let config = config_handle.read();
        std::fs::remove_file(path)?;

        assert_eq!(config.jsonrpc_address, "127.0.0.1:16800");
        let uris = match &config.uri {
            None => panic!(),
            Some(vec) => vec,
        };
        assert_eq!(uris[0], "test_novel_config");

        Ok(())
    }
}
