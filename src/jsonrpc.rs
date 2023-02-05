use crate::error::Error;
use anyhow::Result;
use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::HashMap;
use std::fmt::Formatter;
use std::io::Read;

type Options<'a> = HashMap<&'a str, &'a str>;

#[derive(Debug)]
pub enum JsonRPCError {
    ParseError,
    InvalidRequest,
    MethodNotFound,
    InvalidParams,
    InternalError,
    ServerError,
    OtherError,
    NotStandardResponse,
}

impl std::fmt::Display for JsonRPCError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let msg = match &self {
            Self::ParseError => "parse error",
            Self::InvalidRequest => "invalid request",
            Self::MethodNotFound => "method not found",
            Self::InvalidParams => "invalid params",
            Self::InternalError => "internal error",
            Self::ServerError => "server error",
            Self::OtherError => "application custom error",
            Self::NotStandardResponse => "server returns non-standard response",
        };
        write!(f, "{msg}")
    }
}

impl std::error::Error for JsonRPCError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        None
    }
}

enum JsonRPCMethod {
    Aria2AddUri,
    Aria2GetVersion,
    Aria2TellStatus,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct JsonRPC {
    jsonrpc: String,
    method: Option<String>,
    id: Option<String>,
    params: Option<serde_json::Value>,
}

impl JsonRPC {
    fn new(ua: &str) -> Self {
        let id = ua.to_string();
        JsonRPC {
            jsonrpc: "2.0".to_string(),
            method: None,
            id: Some(id),
            params: None,
        }
    }

    pub fn builder(ua: &str) -> JsonRPCBuilder {
        JsonRPCBuilder::new(ua)
    }

    pub fn to_string(&self) -> Result<String> {
        let ret = serde_json::to_string(&self)?;
        Ok(ret)
    }

    pub fn send(&self, client: &Client, address: &str) -> Result<JsonRPCResponse> {
        let json_string = self.to_string()?;
        let mut client_response = client.post(address).body(json_string).send()?;
        let mut client_response_string = String::new();
        client_response.read_to_string(&mut client_response_string)?;
        let response_value: serde_json::Value = serde_json::from_str(&client_response_string)?;
        let method = self.get_method();
        let ret = JsonRPCResponse {
            value: response_value,
            method,
        };
        Ok(ret)
    }

    fn get_method(&self) -> JsonRPCMethod {
        if let Some(method) = &self.method {
            match method.as_str() {
                "aria2.addUri" => JsonRPCMethod::Aria2AddUri,
                "aria2.getVersion" => JsonRPCMethod::Aria2GetVersion,
                "aria2.tellStatus" => JsonRPCMethod::Aria2TellStatus,
                _ => panic!("unreachable match arm for json rpc method"),
            }
        } else {
            panic!("impossible null method")
        }
    }
}

#[derive(Debug)]
pub struct JsonRPCBuilder {
    inner: JsonRPC,
    available: bool,
}

impl JsonRPCBuilder {
    pub fn new(ua: &str) -> Self {
        let inner = JsonRPC::new(ua);
        Self {
            inner,
            available: false,
        }
    }

    pub fn build(self) -> Result<JsonRPC> {
        if !self.available {
            return Err(anyhow::Error::from(Error::JsonRPCNotReady));
        }
        Ok(self.inner)
    }

    fn parse_token(secret: Option<String>) -> String {
        match secret {
            Some(s) => format!("token:{s}"),
            None => "token:".to_string(),
        }
    }

    pub fn aria2_add_uri(mut self, secret: Option<String>, uri: &str) -> Self {
        let method = "aria2.addUri".to_string();
        let secret = Self::parse_token(secret);
        let params = json!([secret, vec![uri]]);
        self.complete_method(method, params);
        self
    }

    pub fn aria2_get_version(mut self, secret: Option<String>) -> Self {
        let method = "aria2.getVersion".to_string();
        let secret = Self::parse_token(secret);
        let params = json!([secret]);
        self.complete_method(method, params);
        self
    }

    pub fn aria2_tell_status(mut self, secret: Option<String>, gid: String) -> Self {
        let method = "aria2.tellStatus".to_string();
        let secret = Self::parse_token(secret);
        let params = json!([secret, gid, ["status"]]);
        self.complete_method(method, params);
        self
    }

    fn complete_method(&mut self, method: String, params: serde_json::Value) {
        self.inner.method = Some(method);
        self.inner.params = Some(params);
        self.available = true;
    }
}

pub struct JsonRPCResponse {
    value: serde_json::Value,
    method: JsonRPCMethod,
}

impl JsonRPCResponse {
    pub fn unwrap_response(self) -> Result<HashMap<String, String>> {
        if let Some(v) = &self.value.get("error") {
            let code: i32 = v.get("code").unwrap().to_string().parse().unwrap();
            let error = match code {
                -32700 => JsonRPCError::ParseError,
                -32600 => JsonRPCError::InvalidRequest,
                -32601 => JsonRPCError::MethodNotFound,
                -32602 => JsonRPCError::InvalidParams,
                -32603 => JsonRPCError::InternalError,
                -32099..=-32000 => JsonRPCError::ServerError,
                _ => JsonRPCError::OtherError,
            };
            return Err(anyhow::Error::from(error));
        }

        if let Some(v) = &self.value.get("result") {
            return match &self.method {
                JsonRPCMethod::Aria2GetVersion => {
                    let key = "version".to_string();
                    let value = v.get("version").unwrap().to_string();
                    let ret = HashMap::from([(key, value)]);
                    Ok(ret)
                }
                JsonRPCMethod::Aria2AddUri => {
                    let key = "gid".to_string();
                    let value = v.as_str().unwrap().to_string();
                    let ret = HashMap::from([(key, value)]);
                    Ok(ret)
                }
                JsonRPCMethod::Aria2TellStatus => {
                    let key = "status".to_string();
                    let unsafe_string = v.get("status").unwrap().to_string();
                    let value = Self::trim_matches(unsafe_string, '"');
                    println!("{value}");
                    let ret = HashMap::from([(key, String::new())]);
                    Ok(ret)
                }
            };
        }

        Err(anyhow::Error::from(JsonRPCError::NotStandardResponse))
    }

    fn trim_matches(str: String, pat: char) -> String {
        str.trim_matches(pat).to_string()
    }
}
