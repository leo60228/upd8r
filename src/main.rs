#![feature(never_type)]

use anyhow::Result;
use std::{panic, process};
use upd8r::*;

fn main() -> Result<!> {
    let orig_hook = panic::take_hook();
    panic::set_hook(Box::new(move |panic_info| {
        orig_hook(panic_info);
        process::exit(1);
    }));

    let discord = discord::start()?;
    let mastodon = mastodon::start().unwrap();
    let senders = vec![discord, mastodon];
    event_loop::event_loop(senders)
}
