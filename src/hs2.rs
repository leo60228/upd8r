use super::*;
use anyhow::{anyhow, bail, Context, Result};
use chrono::naive::NaiveDate;
use rss::{Channel, Item};
use scraper::{Html, Selector};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use url::Url;

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
                let id = self
                    .title()
                    .ok_or_else(|| anyhow!("Homestuck^2 update missing page!"))?
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

pub struct Hs2Feed(pub Vec<Update>);

fn hash(x: impl Hash) -> u64 {
    let mut hasher = DefaultHasher::new();
    x.hash(&mut hasher);
    hasher.finish()
}

impl Feed for Hs2Feed {
    type Item = Update;

    fn updates(&self) -> &[Update] {
        &self.0
    }

    fn fetch(media: &Media) -> Result<Self> {
        match media {
            &Media::Homestuck2 => {
                let channel = Channel::from_url("https://homestuck2.com/story/rss")
                    .context("Failed to fetch RSS feed")?;
                let mut items: Vec<_> = channel.items().iter().rev().cloned().collect();
                items.dedup_by_key(|item| hash(item.pub_date()));
                let updates = items
                    .into_iter()
                    .rev()
                    .map(|x| x.into_update(media))
                    .collect::<Result<Vec<_>>>()?;
                Ok(Self(updates))
            }
            &Media::Homestuck2Bonus => {
                let text = attohttpc::get("https://homestuck2.com/bonus")
                    .send()?
                    .text()?;
                let doc = Html::parse_document(&text);
                let update_selector = Selector::parse(".type-center > p").unwrap();
                let a_selector = Selector::parse("a").unwrap();
                let updates: Vec<Update> = doc
                    .select(&update_selector)
                    .filter_map(|update| {
                        let date = update.text().next()?.trim_end_matches(" - ");
                        let a = update.select(&a_selector).next()?;
                        let title = a.inner_html();

                        let rel = a.value().attr("href")?;
                        let base = Url::parse("https://homestuck2.com/bonus").unwrap();
                        let abs = base.join(rel).ok()?;
                        let link = abs.as_str().to_string();

                        let mut date_segments = date.split('/');
                        let m = date_segments.next()?.parse().ok()?;
                        let d = date_segments.next()?.parse().ok()?;
                        let y = date_segments.next()?.parse().ok()?;

                        let date = NaiveDate::from_ymd(y, m, d);
                        let datetime = date.and_hms(0, 0, 0);
                        let id = datetime.timestamp() as _;

                        Some(Update {
                            id,
                            title,
                            link,
                            media: media.clone(),
                            show_id: false,
                        })
                    })
                    .collect();
                Ok(Self(updates))
            }
            _ => bail!("{} isn't Homestuck^2!", media),
        }
    }
}
