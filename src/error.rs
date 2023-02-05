use crate::jsonrpc::JsonRPCError;
use std::fmt::Formatter;

#[derive(Debug)]
pub enum Error {
    BadTorrentLink,
    JsonRPCNotReady,
    RPCServerError(JsonRPCError),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let msg = match &self {
            Self::BadTorrentLink => "can not find torrent link".to_string(),
            Self::JsonRPCNotReady => "jsonrpc not ready".to_string(),
            Self::RPCServerError(e) => format!("{e}"),
        };
        write!(f, "{msg}")
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match &self {
            Self::BadTorrentLink => None,
            Self::JsonRPCNotReady => None,
            Self::RPCServerError(e) => Some(e),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[should_panic(expected = "can not find torrent link")]
    fn new_error() {
        let error = Error::BadTorrentLink;
        panic!("{error}")
    }
}
