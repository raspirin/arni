use std::{
    fs::File,
    io::{self, Write},
    path::Path,
    time::SystemTime,
};

use anyhow::{Result, Context};
use serde::{Deserialize, Serialize};

use super::SyncFile;

pub struct History<'a> {
    modified_time: SystemTime,
    path: &'a Path,
    inner: SerdeHistory,
    delta: Vec<String>,
}

impl<'a> History<'a> {
    pub fn new(path: &'a str) -> Result<Self> {
        let path = Path::new(path);
        let inner = if path.exists() {
            let file = File::open(path).with_context(|| "Fail to open path.")?;
            let file = io::read_to_string(file).with_context(|| "Fail to read on disk history file.")?;
            toml::from_str(&file).with_context(|| "Fail to parse history file.")?
        } else {
            let mut file = File::create(path).with_context(|| "Fail to create history file.")?;
            let ret = SerdeHistory::default();
            file.write_all(toml::to_string_pretty(&ret)?.as_bytes())?;
            ret
        };
        let modified_time = path.metadata()?.modified()?;

        Ok(Self {
            modified_time,
            path,
            inner,
            delta: vec![],
        })
    }

    pub fn query(&self, guid: &str) -> bool {
        let guid = guid.to_string();
        self.inner.downloaded.contains(&guid) || self.delta.contains(&guid)
    }

    pub fn push(&mut self, guid: &str) {
        self.delta.push(guid.to_string())
    }
}

impl SyncFile for History<'_> {
    fn modified_time(&self) -> &std::time::SystemTime {
        &self.modified_time
    }

    fn path(&self) -> &std::path::Path {
        self.path
    }

    fn merge(&mut self, on_disk: String) -> Result<()> {
        let on_disk = toml::from_str::<SerdeHistory>(&on_disk)?;
        self.inner = on_disk;
        self.inner.downloaded.append(&mut self.delta);
        self.delta.clear();

        Ok(())
    }

    fn write_back(&mut self) -> Result<()> {
        let mut file = File::create(self.path)?;
        file.write_all(toml::to_string_pretty(&self.inner)?.as_bytes())?;
        self.modified_time = self.path.metadata()?.modified()?;
        Ok(())
    }
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct SerdeHistory {
    downloaded: Vec<String>,
}
