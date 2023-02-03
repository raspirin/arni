use std::fmt::Formatter;

#[derive(Debug)]
pub enum Error {
    BadTorrentLink,
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let msg = match &self {
            Self::BadTorrentLink => "can not find torrent link",
        };
        write!(f, "{msg}")
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match &self {
            Self::BadTorrentLink => None
        }
    }
}

