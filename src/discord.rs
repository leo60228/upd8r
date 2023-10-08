use anyhow::{anyhow, Context as _, Result};
use serenity::{model::gateway::Ready, prelude::*};
use std::thread;

#[allow(clippy::unreadable_literal)]
pub const CHANNEL_ID: u64 = 688125787349712975;

struct Handler(crossbeam_channel::Receiver<String>);

impl Handler {
    fn event_loop(&self, ctx: &Context) -> Result<!> {
        let channel = ctx
            .http
            .get_channel(CHANNEL_ID)?
            .guild()
            .ok_or_else(|| anyhow!("Channel {} not a guild channel!", CHANNEL_ID))?;

        loop {
            let upd8_str = self.0.recv()?;
            channel
                .read()
                .say(&ctx.http, upd8_str)
                .context("Failed to send message")?;
        }
    }
}

impl EventHandler for Handler {
    fn ready(&self, ctx: Context, ready: Ready) {
        println!("[Discord] Connected as {}", ready.user.name);
        self.event_loop(&ctx).unwrap();
    }
}

pub fn start() -> Result<crossbeam_channel::Sender<String>> {
    let token = dotenv::var("DISCORD_TOKEN")?;
    let (s, r) = crossbeam_channel::unbounded();
    let mut client = Client::new(&token, Handler(r))?;
    thread::spawn(move || {
        client.start().unwrap();
    });

    Ok(s)
}
