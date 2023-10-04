use crate::persist::Persist;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct HistoryMeta {
    #[serde(rename = "downloaded")]
    pub is_downloaded: bool,
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
                    },
                );
                false
            }
        }
    }

    pub fn get_metadata_mut(&mut self, guid: &str) -> &mut HistoryMeta {
        self.inner.get_mut(guid).unwrap()
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_empty_history() {
        let history = History::new_empty();
        let inner = &history.inner;

        assert_eq!(inner, &HashMap::new());
    }

    #[test]
    fn test_history_from_str() {
        let s = r#"[inner."test_guid"]
downloaded = false"#;
        let history = History::from_str(s).unwrap();
        assert!(!history.inner.get("test_guid").unwrap().is_downloaded);
    }

    #[test]
    #[should_panic]
    fn test_history_invalid_str() {
        let s = r#"[inner."test_guid"]
is_download = 1"#;
        let _ = History::from_str(s).unwrap();
    }

    #[test]
    fn test_history_to_string() {
        let mut history = History::new();
        history.inner.insert(
            String::from("test guid 1"),
            HistoryMeta {
                is_downloaded: false,
            },
        );
        let toml = r#"[inner."test guid 1"]
downloaded = false
"#;
        let s = history.to_string().unwrap();

        assert_eq!(toml, s);
    }

    #[test]
    fn test_query_download() {
        let mut history = History::new_empty();

        assert!(!history.query_downloaded("1"));
        assert!(!history.inner.get("1").unwrap().is_downloaded);
        assert!(!history.query_downloaded("1"));
    }
}
