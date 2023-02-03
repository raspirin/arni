use anyhow::Result;
use arni::{Context, error::Error};

struct Episode {
    guid: String,
    title: Option<String>,
    torrent_link: String,
}

impl Episode {
    fn new(guid: String, title: Option<String>, torrent_link: String) -> Self {
        Self {
            guid,
            title,
            torrent_link,
        }
    }
}

struct EpisodeList {
    episodes: Vec<Episode>
}

impl EpisodeList {
    fn new() -> Self {
        Self {
            episodes: vec![]
        }
    }

    fn push(&mut self, item: &rss::Item) -> Result<()> {
        let torrent_link = match item.enclosure() {
            Some(enclosure) => enclosure.url().to_string(),
            None => return Err(anyhow::Error::from(Error::BadTorrentLink)),
        };
        let guid = match item.guid() {
            Some(guid) => guid.value().to_string(),
            // if there is None, the function will return before this unwrap()
            None => item.enclosure().unwrap().url().to_string(),
        };
        let title = item.title().map(|title| title.to_string());

        let episode = Episode::new(guid, title, torrent_link);
        self.episodes.push(episode);
        Ok(())
    }
}

fn main() -> Result<()> {
    // init basic context
    let default_config_path = "config.toml";
    let mut context = Context::new(default_config_path)?;
    context.prepare_channels()?;

    // init items
    let mut episodes = EpisodeList::new();

    for channel in context.channels.iter() {
        for item in channel.items().iter() {
            episodes.push(item)?;
        }
    }

    Ok(())
}
