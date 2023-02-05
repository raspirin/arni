use anyhow::Result;
use serde::{Deserialize, Serialize};
use crate::persist::Persist;

#[derive(Serialize, Deserialize)]
pub struct Config {
    pub uri: Option<Vec<String>>,
    pub file: Option<Vec<String>>,
}

impl Config {
    fn new() -> Self {
        Self {
            uri: Some(vec![]),
            file: Some(vec![]),
        }
    }
}

impl Persist for Config {
    fn new_empty() -> Self {
        Config::new()
    }

    fn from_str(s: &str) -> Result<Self> {
        let ret: Config = toml::from_str(s)?;
        Ok(ret)
    }

    fn to_string(&self) -> Result<String> {
        let ret = toml::to_string(&self)?;
        Ok(ret)
    }
}

#[cfg(test)]
mod tests {
    use std::fs::File;
    use std::io::{Read, Write};
    use super::*;

    #[test]
    fn test_new_config() {
        let config = Config::new();
        assert_eq!(config.uri, Some(vec![]));
        assert_eq!(config.file, Some(vec![]));
        let str = config.to_string().unwrap();
        let r = r#"uri = []
file = []
"#;
        assert_eq!(str, r);
    }

    #[test]
    fn test_from_str() {
        let s = r#"uri = ["test_uri"]"#;
        let config = Config::from_str(s).unwrap();
        assert_eq!(config.file, None);
        assert_eq!(config.uri, Some(vec!["test_uri".to_string()]));
    }

    #[test]
    #[should_panic]
    fn test_from_invalid_str() {
        let s = r#"uri = [1]"#;
        let _config = Config::from_str(s).unwrap();
    }

    #[test]
    fn test_to_string() {
        let c = Config {
            file: Some(vec!["testfile".to_string()]),
            uri: Some(vec![]),
        };
        let s = c.to_string().unwrap();
        let r = r#"uri = []
file = ["testfile"]
"#;
        assert_eq!(s, r);
    }

    #[test]
    fn test_write_to_disk() {
        let config = Config::new();
        let test_file = "test_write_to_disk_config.toml";
        config.write_to_disk(test_file).unwrap();
        std::fs::remove_file(test_file).unwrap();
    }

    #[test]
    fn test_load_disk_config() -> Result<()> {
        let file_path = "test_load_disk_config_config.toml";
        let mut file = File::create(file_path)?;
        let toml = r#"uri = []"#;
        file.write_all(toml.as_bytes())?;
        let config = Config::load_from_disk(file_path)?;
        std::fs::remove_file(file_path)?;

        assert_eq!(config.uri, Some(vec![]));
        assert_eq!(config.file, None);
        Ok(())
    }

    #[test]
    fn test_init_config_on_disk() -> Result<()> {
        let file_path = "test_init_config_on_disk";
        let mut file = File::create(file_path)?;
        let toml = r#"uri = []"#;
        file.write_all(toml.as_bytes())?;
        let config = Config::load(file_path)?;
        std::fs::remove_file(file_path)?;

        assert_eq!(config.uri, Some(vec![]));
        assert_eq!(config.file, None);
        Ok(())
    }

    #[test]
    fn test_init_config_from_memory() -> Result<()> {
        let path = "test_init_config_from_memory";
        let config = Config::load(path)?;
        assert_eq!(config.uri, Some(vec![]));
        assert_eq!(config.file, Some(vec![]));

        let mut file = File::open(path)?;
        let toml = config.to_string()?;
        let mut on_disk = String::new();
        file.read_to_string(&mut on_disk)?;
        std::fs::remove_file(path)?;
        assert_eq!(toml, on_disk);

        Ok(())
    }
}
