use super::*;
use anyhow::{bail, Result};
use scraper::{Html, Selector};

pub struct SbFeed(pub Vec<Update>);

impl Feed for SbFeed {
    type Item = Update;

    fn updates(&self) -> &[Update] {
        &self.0
    }

    fn fetch(media: &Media) -> Result<Self> {
        match media {
            &Media::Sb20020 => {
                let text = attohttpc::get(
                    "https://www.sbnation.com/secret-base/21410129/20020/chapters-index",
                )
                .send()?
                .text()?;
                let doc = Html::parse_document(&text);
                let update_selector =
                    Selector::parse("#chapters-index ul.chapters-list li a").unwrap();
                let mut updates: Vec<Update> = doc
                    .select(&update_selector)
                    .filter_map(|a| {
                        let title = a.inner_html().split(": ").nth(1)?.to_string();
                        let rel = a.value().attr("href")?;
                        let link =
                            "https://www.sbnation.com/secret-base/21410129/20020".to_string() + rel;
                        let id = a
                            .inner_html()
                            .split(&[' ', ':'] as &[_])
                            .nth(1)?
                            .parse()
                            .ok()?;

                        Some(Update {
                            id,
                            title,
                            link,
                            media: media.clone(),
                            show_id: true,
                        })
                    })
                    .collect();
                updates.sort_by(|a, b| b.id.cmp(&a.id));
                Ok(Self(updates))
            }
            _ => bail!("{} isn't 20020!", media),
        }
    }
}
