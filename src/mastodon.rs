use elefren::helpers::{cli, toml};
use elefren::prelude::*;
use elefren::Language;
use std::error::Error;
use std::thread;

fn get_mastodon_data() -> Result<Mastodon, Box<dyn Error>> {
    if let Ok(data) = toml::from_file("mastodon-data.toml") {
        Ok(Mastodon::from(data))
    } else {
        register()
    }
}

fn register() -> Result<Mastodon, Box<dyn Error>> {
    let website = "https://60228.dev";
    let registration = Registration::new(website.trim())
        .client_name("upd8r")
        .scopes(Scopes::all())
        .website("https://github.com/leo60228/upd8r")
        .build()?;
    let mastodon = cli::authenticate(registration)?;

    // Save app data for using on the next run.
    toml::to_file(&*mastodon, "mastodon-data.toml")?;

    Ok(mastodon)
}

fn event_loop(
    mastodon: Mastodon,
    chan: crossbeam_channel::Receiver<String>,
) -> Result<!, Box<dyn Error>> {
    let acc = mastodon.verify_credentials()?;
    println!("[Mastodon] Connected as {}", acc.username);

    loop {
        let upd8_str = chan.recv()?;
        let status = StatusBuilder::new()
            .status(upd8_str)
            .language(Language::Eng)
            .build()?;
        mastodon.new_status(status)?;
    }
}

pub fn start() -> Result<crossbeam_channel::Sender<String>, Box<dyn Error>> {
    let mastodon = get_mastodon_data()?;

    let (s, r) = crossbeam_channel::unbounded();
    thread::spawn(move || {
        event_loop(mastodon, r).unwrap();
    });

    Ok(s)
}
