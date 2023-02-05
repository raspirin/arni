use crate::persist::Persist;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Debug)]
pub struct HistoryMeta {
    pub is_downloaded: bool,
    pub gid: Option<u64>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct History {
    inner: HashMap<String, HistoryMeta>,
}

impl History {
    fn new() -> Self {
        Self {
            inner: HashMap::new(),
        }
    }

    pub fn query_downloaded(&mut self, guid: &str) -> bool {
        let guid = guid.to_string();
        match self.inner.get(&guid) {
            Some(meta) => meta.is_downloaded,
            None => {
                self.inner.insert(
                    guid,
                    HistoryMeta {
                        is_downloaded: false,
                        gid: None,
                    },
                );
                false
            }
        }
    }
}

impl Persist for History {
    fn new_empty() -> Self {
        History::new()
    }

    fn from_str(s: &str) -> Result<Self> {
        let ret: History = toml::from_str(s)?;
        Ok(ret)
    }

    fn to_string(&self) -> Result<String> {
        let ret = toml::to_string(&self)?;
        Ok(ret)
    }
}
