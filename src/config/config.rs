use anyhow::{Result, Context};
use std::{
    fs::File,
    io::{self, Write},
    path::Path,
    time::SystemTime,
};

use serde::{Deserialize, Serialize};

use super::SyncFile;

pub struct Config<'a> {
    modified_time: SystemTime,
    path: &'a Path,
    inner: SerdeConfig,
}

impl<'a> Config<'a> {
    pub fn new(path: &'a str) -> Result<Self> {
        let path = Path::new(path);
        let inner = if path.exists() {
            let file = File::open(path).with_context(|| "Fail to open path.")?;
            let file = io::read_to_string(file).with_context(|| "Fail to read on disk config file.")?;
            toml::from_str(&file).with_context(|| "Fail to parse config file.")?
        } else {
            let mut file = File::create(path).with_context(|| "Fail to create config file.")?;
            let ret = SerdeConfig::default();
            file.write_all(toml::to_string_pretty(&ret).with_context(|| "Fail to write new config file back.")?.as_bytes())?;
            ret
        };
        let modified_time = path.metadata()?.modified()?;

        Ok(Self {
            modified_time,
            path,
            inner,
        })
    }

    pub fn aria2_address(&self) -> &String {
        &self.inner.aria2_address
    }

    pub fn url(&self) -> &Option<Vec<String>> {
        &self.inner.url
    }

    pub fn file(&self) -> &Option<Vec<String>> {
        &self.inner.file
    }
}

impl SyncFile for Config<'_> {
    fn modified_time(&self) -> &std::time::SystemTime {
        &self.modified_time
    }

    fn path(&self) -> &std::path::Path {
        self.path
    }

    fn merge(&mut self, on_disk: String) -> Result<()> {
        let on_disk = toml::from_str::<SerdeConfig>(&on_disk)?;
        self.inner = on_disk;

        Ok(())
    }

    fn write_back(&mut self) -> Result<()> {
        let mut file = File::create(self.path).with_context(|| "Fail to write back.")?;
        let toml = toml::to_string_pretty(&self.inner)?;
        file.write_all(toml.as_bytes())?;
        self.modified_time = self.path.metadata()?.modified()?;
        Ok(())
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SerdeConfig {
    pub aria2_address: String,
    pub url: Option<Vec<String>>,
    pub file: Option<Vec<String>>,
}

impl Default for SerdeConfig {
    fn default() -> Self {
        Self {
            aria2_address: "127.0.0.1:6800".to_string(),
            url: None,
            file: None,
        }
    }
}
