use crate::error::Error;

#[derive(Debug, PartialEq, Eq)]
pub enum DownloadStatus {
    /// Waiting for sending to aria2
    Waiting,
    /// Sent to aria2
    Sent,
    /// Finished downloading
    Done,
    /// Something went wrong on the aria2 side
    Error,
}

pub struct Episode {
    pub guid: String,
    pub title: Option<String>,
    pub torrent_link: String,
    pub gid: Option<String>,
    pub download_status: DownloadStatus,
}

impl Episode {
    pub fn new(guid: String, title: Option<String>, torrent_link: String) -> Self {
        Self {
            guid,
            title,
            torrent_link,
            gid: None,
            download_status: DownloadStatus::Waiting,
        }
    }

    pub fn is_waiting(&self) -> bool {
        self.download_status == DownloadStatus::Waiting
    }

    pub fn is_sent(&self) -> bool {
        self.download_status == DownloadStatus::Sent
    }

    pub fn is_done(&self) -> bool {
        self.download_status == DownloadStatus::Done
    }

    pub fn set_download_status(&mut self, status: &str) -> Result<(), Error> {
        self.download_status = match status {
            "active" | "waiting" | "paused" => DownloadStatus::Sent,
            "error" => DownloadStatus::Error,
            "complete" | "removed" => DownloadStatus::Done,
            _ => return Err(Error::ImpossibleEpisodeState),
        };

        Ok(())
    }

    pub fn set_sent(&mut self) {
        self.download_status = DownloadStatus::Sent;
    }

    pub fn gid(&self) -> Result<String, Error> {
        match &self.gid {
            Some(gid) => Ok(gid.to_string()),
            None => Err(Error::ImpossibleEpisodeState),
        }
    }
}

impl PartialEq for Episode {
    fn eq(&self, other: &Self) -> bool {
        self.guid == other.guid
    }
}

impl TryFrom<rss::Item> for Episode {
    type Error = Error;

    fn try_from(value: rss::Item) -> std::result::Result<Self, Self::Error> {
        let torrent_link = match value.enclosure() {
            Some(enclosure) => enclosure.url().to_string(),
            None => return Err(Error::BadTorrentLink),
        };
        let guid = match value.guid() {
            Some(guid) => guid.value().to_string(),
            // if there is None, the function will return before this unwrap()
            None => value.enclosure().unwrap().url().to_string(),
        };
        let title = value.title().map(|title| title.to_string());
        Ok(Self::new(guid, title, torrent_link))
    }
}
