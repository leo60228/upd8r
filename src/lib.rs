#![feature(atomic_min_max, never_type)]

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
pub mod hs2;
pub mod steam;

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Display)]
pub enum Media {
    #[display(fmt = "Homestuck^2")]
    Homestuck2,
    #[display(fmt = "Pesterquest")]
    Pesterquest,
    #[display(fmt = "Hiveswap Act 2")]
    HiveswapAct2,
}

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
        static LATEST_PQ: AtomicU64 = AtomicU64::new(0);
        static LOAD_PQ: Once = Once::new();
        static LATEST_HIVESWAP_A2: AtomicU64 = AtomicU64::new(0);
        static LOAD_HIVESWAP_A2: Once = Once::new();
        let (latest, load, filename) = match self.media {
            Media::Homestuck2 => (&LATEST_HS2, &LOAD_HS2, "latest_hs2_upd8"),
            Media::Pesterquest => (&LATEST_PQ, &LOAD_PQ, "latest_pq_upd8"),
            Media::HiveswapAct2 => (
                &LATEST_HIVESWAP_A2,
                &LOAD_HIVESWAP_A2,
                "latest_hiveswap_a2_upd8",
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

pub trait Feed: Sized {
    type Item: IntoUpdate;

    fn updates(&self) -> &[Self::Item];
    fn fetch(media: &Media) -> Result<Self>;
}

macro_rules! upd8 {
    ($feed:ty, $media:ident) => {
        <$feed>::fetch($media)?.updates()[0].into_update($media)?
    };
}

pub fn check_for_update(media: &Media) -> Result<Option<Update>> {
    let update = match media {
        Media::Homestuck2 => upd8!(hs2::Hs2Feed, media),
        Media::Pesterquest | Media::HiveswapAct2 => upd8!(steam::AppNews, media),
    };
    if update.latest()? {
        Ok(Some(update))
    } else {
        Ok(None)
    }
}
