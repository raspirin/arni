use crate::jsonrpc::JsonRPCError;
use std::fmt::Formatter;

#[derive(Debug)]
pub enum Error {
    BadTorrentLink,
    JsonRPCNotReady,
    ImpossibleEpisodeState,
    RPCServerError(JsonRPCError),
    Aria2ConnectionError,
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let msg = match &self {
            Self::BadTorrentLink => "can not find torrent link".to_string(),
            Self::JsonRPCNotReady => "jsonrpc not ready".to_string(),
            Self::ImpossibleEpisodeState => "Impossible Episode State".to_string(),
            Self::Aria2ConnectionError => "Can't connect to aria2".to_string(),
            Self::RPCServerError(e) => format!("{e}"),
        };
        write!(f, "{msg}")
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match &self {
            Self::RPCServerError(e) => Some(e),
            _ => None,
        }
    }
}

impl From<JsonRPCError> for Error {
    fn from(value: JsonRPCError) -> Self {
        Self::RPCServerError(value)
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
