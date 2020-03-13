use super::*;
use anyhow::{anyhow, ensure, Context, Result};
use rss::{Channel, Item};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

impl IntoUpdate for Item {
    fn into_update(&self, media: &Media) -> Result<Update> {
        let link = self
            .link()
            .ok_or_else(|| anyhow!("{} update missing link!", media))?;
        let (title, id) = match media {
            Media::Homestuck2 => {
                let title = self
                    .description()
                    .ok_or_else(|| anyhow!("Homestuck^2 update missing title!"))?
                    .to_string();
                let id = link
                    .trim_start_matches("https://www.homestuck2.com/story/")
                    .parse()
                    .context("Couldn't get page number from Homestuck^2 update!")?;
                (title, id)
            }
            _ => return Err(anyhow!("{} not an RSS feed!", media)),
        };
        Ok(Update {
            id,
            title,
            link: link.to_string(),
            media: media.clone(),
            show_id: true,
        })
    }
}

pub struct Hs2Feed(pub Vec<Item>);

fn hash(x: impl Hash) -> u64 {
    let mut hasher = DefaultHasher::new();
    x.hash(&mut hasher);
    hasher.finish()
}

impl Feed for Hs2Feed {
    type Item = Item;

    fn updates(&self) -> &[Item] {
        &self.0
    }

    fn fetch(media: &Media) -> Result<Self> {
        ensure!(media == &Media::Homestuck2, "{} isn't Homestuck^2!", media);
        let channel = Channel::from_url("https://homestuck2.com/story/rss")
            .context("Failed to fetch RSS feed")?;
        let mut items: Vec<_> = channel.items().iter().rev().cloned().collect();
        items.dedup_by_key(|item| hash(item.pub_date()));
        items.reverse();
        Ok(Self(items))
    }
}
