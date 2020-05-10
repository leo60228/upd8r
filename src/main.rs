#![feature(never_type)]

use anyhow::Result;
use upd8r::*;

fn main() -> Result<!> {
    let discord = discord::start()?;
    let mastodon = mastodon::start().unwrap();
    let senders = vec![discord, mastodon];
    event_loop::event_loop(senders)
}
