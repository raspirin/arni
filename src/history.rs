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
        let a: u64 = 0x2089b05ecca3d829;
        file.write_all(self.serialize().as_bytes()).unwrap();
    }
}
