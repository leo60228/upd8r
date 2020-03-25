use super::*;
use anyhow::{anyhow, Context as _, Result};
use serenity::{model::gateway::Ready, prelude::*};
use std::thread;
use std::time::Duration;

#[allow(clippy::unreadable_literal)]
pub const CHANNEL_ID: u64 = 688125787349712975;

struct Handler;

impl EventHandler for Handler {
    fn ready(&self, ctx: Context, ready: Ready) {
        println!("Connected as {}", ready.user.name);
        event_loop(&ctx).unwrap();
    }
}

pub fn start() -> Result<()> {
    let token = dotenv::var("DISCORD_TOKEN")?;
    let mut client = Client::new(&token, Handler)?;
    client.start()?;

    Ok(())
}

pub fn event_loop(ctx: &Context) -> Result<!> {
    let channel = ctx
        .http
        .get_channel(CHANNEL_ID)?
        .guild()
        .ok_or_else(|| anyhow!("Channel {} not a guild channel!"))?;

    loop {
        for media in &[
            Media::Homestuck2,
            Media::PQSteam,
            Media::HiveswapAct2,
            Media::PQTwitter,
            Media::Homestuck2Bonus,
        ] {
            println!("Checking for {} updates...", media);
            match check_for_update(media) {
                Ok(Some(upd8)) => {
                    let upd8_str = upd8.to_string();
                    println!("{}", upd8_str);
                    channel
                        .read()
                        .say(&ctx.http, upd8_str)
                        .context("Failed to send message")?;
                }
                Ok(None) => println!("No new update."),
                Err(err) => eprintln!("Error: {}", err),
            }
        }
        thread::sleep(Duration::from_secs(60));
    }
}
