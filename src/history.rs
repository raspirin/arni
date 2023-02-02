use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::io::Write;

#[derive(Deserialize, Serialize)]
pub struct History {
    pub cache: HashMap<String, bool>,
}

impl History {
    pub fn serialize(&self) -> String {
        toml::to_string(self).unwrap()
    }

    pub fn archive(&self, path: &str) {
        let mut file = std::fs::File::create(path).unwrap();
        file.write_all(self.serialize().as_bytes()).unwrap();
    }
}
