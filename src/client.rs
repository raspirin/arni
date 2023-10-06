use std::io::Read;

use anyhow::Result;

use crate::jsonrpc::{JsonRPC, JsonRPCResponse};

pub struct Client {
    client: reqwest::blocking::Client,
}

impl Client {
    pub fn new() -> Result<Self> {
        Self::with_ua(concat!(
            env!("CARGO_PKG_NAME"),
            "/",
            env!("CARGO_PKG_VERSION")
        ))
    }

    pub fn with_ua(ua: &str) -> Result<Self> {
        let client = reqwest::blocking::Client::builder()
            .user_agent(ua)
            .build()?;
        Ok(Self { client })
    }

    pub fn inner(&self) -> &reqwest::blocking::Client {
        &self.client
    }

    pub fn inner_mut(&mut self) -> &mut reqwest::blocking::Client {
        &mut self.client
    }

    pub fn dry_send(&self, _address: &str, jsonrpc: JsonRPC) -> Result<String> {
        let _method = jsonrpc.get_method();
        let jsonrpc = jsonrpc.to_string()?;
        Ok(jsonrpc)
    }

    pub fn send(&mut self, address: &str, jsonrpc: JsonRPC) -> Result<JsonRPCResponse> {
        let method = jsonrpc.get_method();
        let jsonrpc = jsonrpc.to_string()?;
        let mut response = self.client.post(address).body(jsonrpc).send()?;
        let mut response_value = String::new();
        response.read_to_string(&mut response_value)?;
        let response_value: serde_json::Value = serde_json::from_str(&response_value)?;
        Ok(JsonRPCResponse {
            value: response_value,
            method,
        })
    }
}
