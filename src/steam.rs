use super::*;
use anyhow::{anyhow, Context, Result};
use serde::{Deserialize, Serialize};

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize, Debug, Hash)]
pub struct AppNews {
    #[serde(rename = "appid")]
    pub app_id: u32,
    #[serde(rename = "newsitems")]
    pub news_items: Vec<NewsItem>,
    pub count: usize,
}

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize, Debug, Hash)]
pub struct NewsItem {
    #[serde(with = "serde_with::rust::display_fromstr")]
    pub gid: u128,
    pub title: String,
    pub url: String,
    pub is_external_url: bool,
    pub author: String,
    pub contents: String,
    #[serde(rename = "feedlabel")]
    pub feed_label: String,
    pub date: u64,
    #[serde(rename = "feedname")]
    pub feed_name: String,
    pub feed_type: usize,
    #[serde(rename = "appid")]
    pub app_id: u32,
    #[serde(default)]
    pub tags: Vec<String>,
}

impl IntoUpdate for NewsItem {
    fn into_update(&self, media: &Media) -> Result<Update> {
        Ok(Update {
            id: self.date,
            title: self.title.clone(),
            link: self.url.clone(),
            media: media.clone(),
            show_id: false,
        })
    }
}

impl Feed for AppNews {
    type Item = NewsItem;

    fn updates(&self) -> &[Self::Item] {
        &self.news_items
    }

    fn fetch(media: &Media) -> Result<Self> {
        let url = match media {
            Media::PQSteam => "https://api.steampowered.com/ISteamNews/GetNewsForApp/v0002/?appid=1144030&count=999999&maxlength=1&format=json",
            Media::HiveswapAct2 => "https://api.steampowered.com/ISteamNews/GetNewsForApp/v0002/?appid=1181840&count=999999&maxlength=1&format=json",
            _ => return Err(anyhow!("{} not on Steam!")),
        };
        #[derive(Deserialize)]
        struct Response {
            #[serde(rename = "appnews")]
            app_news: AppNews,
        }
        let resp = attohttpc::get(url)
            .send()
            .context("Failed to get Steam news")?;
        if resp.is_success() {
            Ok(resp.json::<Response>()?.app_news)
        } else {
            Err(anyhow!("Error from Steam API: {}", resp.status()))
        }
    }
}
