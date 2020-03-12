#![feature(atomic_min_max)]

use anyhow::{anyhow, Context, Result};
use derive_more::Display;
use rss::{Channel, Item};
use std::cmp::{Ordering as CmpOrdering, PartialOrd};
use std::fmt;
use std::fs;
use std::sync::{
    atomic::{AtomicUsize, Ordering as AOrdering},
    Once,
};

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Display)]
pub enum Media {
    #[display(fmt = "Homestuck^2")]
    Homestuck2,
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Update {
    pub id: usize, // monotonically increasing
    pub title: String,
    pub link: String,
    pub media: Media,
}

impl Update {
    pub fn from_item(media: Media, item: &Item) -> Result<Self> {
        let link = item
            .link()
            .ok_or_else(|| anyhow!("{} update missing link!", media))?;
        let (title, id) = match media {
            Media::Homestuck2 => {
                let title = item
                    .description()
                    .ok_or_else(|| anyhow!("Homestuck^2 update missing title!"))?
                    .to_string();
                let id = link
                    .trim_start_matches("https://www.homestuck2.com/story/")
                    .parse()
                    .context("Couldn't get page number from Homestuck^2 update!")?;
                (title, id)
            }
        };
        Ok(Update {
            id,
            title,
            link: link.to_string(),
            media,
        })
    }

    pub fn latest(&self) -> Result<bool> {
        match self.media {
            Media::Homestuck2 => {
                static LATEST_HS2: AtomicUsize = AtomicUsize::new(0);
                static LOAD_HS2: Once = Once::new();
                const FILENAME: &str = "latest_hs2_page";
                LOAD_HS2.call_once(|| {
                    let _: Result<()> = (|| {
                        let file = fs::read_to_string(FILENAME)?;
                        let latest = file.trim().parse()?;
                        LATEST_HS2.store(latest, AOrdering::SeqCst);
                        Ok(())
                    })();
                });
                let is_latest = LATEST_HS2.fetch_max(self.id, AOrdering::SeqCst) < self.id;
                if is_latest {
                    fs::write(FILENAME, self.id.to_string())?;
                }
                Ok(is_latest)
            }
        }
    }
}

impl PartialOrd for Update {
    fn partial_cmp(&self, other: &Self) -> Option<CmpOrdering> {
        self.id.partial_cmp(&other.id)
    }
}

impl fmt::Display for Update {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} upd8 #{}! {}\n{}",
            self.media, self.id, self.title, self.link
        )
    }
}

pub fn hs2_feed() -> Result<Channel> {
    let channel = Channel::from_url("https://homestuck2.com/story/rss")
        .context("Failed to fetch Homestuck^2 RSS")?;
    Ok(channel)
}
