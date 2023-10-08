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
            Media::HiveswapAct2News => "https://api.steampowered.com/ISteamNews/GetNewsForApp/v0002/?appid=1181840&count=999999&maxlength=1&format=json",
            _ => return Err(anyhow!("{:?} not on Steam!", media)),
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

pub struct AppChanges {
    latest: Update,
}

impl Feed for AppChanges {
    type Item = Update;

    fn updates(&self) -> &[Self::Item] {
        std::slice::from_ref(&self.latest)
    }

    fn fetch(media: &Media) -> Result<Self> {
        let app_id = match media {
            Media::HiveswapAct2Steam => 1181840,
            _ => return Err(anyhow!("{:?} not on Steam!", media)),
        };
        let api_url = format!("https://steamappinfo.leo60228.space/productinfo/{}", app_id);
        let steam_url = format!("https://store.steampowered.com/app/{}", app_id);
        #[derive(Deserialize)]
        struct Response {
            #[serde(rename = "changeNumber")]
            change_number: u64,
        }
        let resp = attohttpc::get(api_url)
            .send()
            .context("Failed to get changenumber")?;
        if resp.is_success() {
            Ok(Self {
                latest: Update {
                    id: resp.json::<Response>()?.change_number,
                    title: "".to_string(),
                    link: steam_url,
                    media: *media,
                    show_id: true,
                },
            })
        } else {
            Err(anyhow!("Error from SteamAppinfo: {}", resp.status()))
        }
    }
}
