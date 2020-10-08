use super::*;
use anyhow::Result;
use std::thread;
use std::time::Duration;

pub fn event_loop(senders: Vec<crossbeam_channel::Sender<String>>) -> Result<!> {
    loop {
        for media in MEDIA_LIST {
            println!("[upd8r] Checking for {} updates...", media);
            match check_for_update(media) {
                Ok(Some(upd8)) => {
                    let upd8_str = upd8.to_string();
                    println!("[upd8r] {}", upd8_str);
                    for sender in &senders {
                        sender.send(upd8_str.clone())?;
                    }
                }
                Ok(None) => println!("[upd8r] No new update."),
                Err(err) => eprintln!("[upd8r] Error: {}", err),
            }
        }
        thread::sleep(Duration::from_secs(60));
    }
}
