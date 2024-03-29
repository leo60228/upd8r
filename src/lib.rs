#![feature(never_type)]

use anyhow::Result;
use derive_more::Display;
use std::cmp::{Ordering as CmpOrdering, PartialOrd};
use std::fmt;
use std::fs;
use std::sync::{
    atomic::{AtomicU64, Ordering as AOrdering},
    Once,
};

pub mod discord;
pub mod event_loop;
pub mod hs2;
pub mod mastodon;
pub mod steam;

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Display)]
pub enum Media {
    #[display(fmt = "Homestuck^2")]
    Homestuck2,
    #[display(fmt = "Homestuck^2 bonus")]
    Homestuck2Bonus,
    #[display(fmt = "PesterQuest")]
    PQSteam,
    #[display(fmt = "Hiveswap Act 2 news")]
    HiveswapAct2News,
    #[display(fmt = "Hiveswap Act 2 Steam")]
    HiveswapAct2Steam,
}

pub const MEDIA_LIST: &[Media] = &[
    Media::Homestuck2,
    Media::PQSteam,
    Media::HiveswapAct2News,
    Media::HiveswapAct2Steam,
    //Media::Homestuck2Bonus,
];

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Update {
    pub id: u64, // monotonically increasing
    pub title: String,
    pub link: String,
    pub media: Media,
    pub show_id: bool,
}

impl Update {
    pub fn latest(&self) -> Result<bool> {
        static LATEST_HS2: AtomicU64 = AtomicU64::new(0);
        static LOAD_HS2: Once = Once::new();
        static LATEST_HS2_BONUS: AtomicU64 = AtomicU64::new(0);
        static LOAD_HS2_BONUS: Once = Once::new();
        static LATEST_PQ: AtomicU64 = AtomicU64::new(0);
        static LOAD_PQ: Once = Once::new();
        static LATEST_HIVESWAP_A2_NEWS: AtomicU64 = AtomicU64::new(0);
        static LOAD_HIVESWAP_A2_NEWS: Once = Once::new();
        static LATEST_HIVESWAP_A2_STEAM: AtomicU64 = AtomicU64::new(0);
        static LOAD_HIVESWAP_A2_STEAM: Once = Once::new();
        let (latest, load, filename) = match self.media {
            Media::Homestuck2 => (&LATEST_HS2, &LOAD_HS2, "latest_hs2_upd8"),
            Media::Homestuck2Bonus => (&LATEST_HS2_BONUS, &LOAD_HS2_BONUS, "latest_hs2_bonus_upd8"),
            Media::PQSteam => (&LATEST_PQ, &LOAD_PQ, "latest_pq_upd8"),
            Media::HiveswapAct2News => (
                &LATEST_HIVESWAP_A2_NEWS,
                &LOAD_HIVESWAP_A2_NEWS,
                "latest_hiveswap_a2_news_upd8",
            ),
            Media::HiveswapAct2Steam => (
                &LATEST_HIVESWAP_A2_STEAM,
                &LOAD_HIVESWAP_A2_STEAM,
                "latest_hiveswap_a2_steam_upd8",
            ),
        };
        load.call_once(|| {
            let _: Result<()> = (|| {
                let file = fs::read_to_string(filename)?;
                let parsed = file.trim().parse()?;
                latest.store(parsed, AOrdering::SeqCst);
                Ok(())
            })();
        });
        let is_latest = latest.fetch_max(self.id, AOrdering::SeqCst) < self.id;
        if is_latest {
            fs::write(filename, self.id.to_string())?;
        }
        Ok(is_latest)
    }

    pub fn from<T: IntoUpdate + ?Sized>(item: &T, media: &Media) -> Result<Self> {
        item.into_update(media)
    }
}

impl PartialOrd for Update {
    fn partial_cmp(&self, other: &Self) -> Option<CmpOrdering> {
        self.id.partial_cmp(&other.id)
    }
}

impl fmt::Display for Update {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.show_id {
            write!(
                f,
                "{} upd8 #{}! {}\n{}",
                self.media, self.id, self.title, self.link
            )
        } else {
            write!(f, "{} upd8! {}\n{}", self.media, self.title, self.link)
        }
    }
}

pub trait IntoUpdate {
    fn into_update(&self, media: &Media) -> Result<Update>;
}

impl IntoUpdate for Update {
    fn into_update(&self, _: &Media) -> Result<Update> {
        Ok(self.clone())
    }
}

pub trait Feed: Sized {
    type Item: IntoUpdate;

    fn updates(&self) -> &[Self::Item];
    fn fetch(media: &Media) -> Result<Self>;
}

macro_rules! upd8 {
    ($feed:ty, $media:ident) => {
        if let Some(upd8) = <$feed>::fetch($media)?.updates().get(0) {
            upd8.into_update($media)?
        } else {
            return Ok(None);
        }
    };
}

pub fn check_for_update(media: &Media) -> Result<Option<Update>> {
    let update = match media {
        Media::Homestuck2 | Media::Homestuck2Bonus => upd8!(hs2::Hs2Feed, media),
        Media::PQSteam | Media::HiveswapAct2News => upd8!(steam::AppNews, media),
        Media::HiveswapAct2Steam => upd8!(steam::AppChanges, media),
    };
    if update.latest()? {
        Ok(Some(update))
    } else {
        Ok(None)
    }
}
