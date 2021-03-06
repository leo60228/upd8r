use anyhow::Result;
use upd8r::*;

fn main() -> Result<()> {
    for media in MEDIA_LIST {
        println!("Checking for {} updates...", media);
        match check_for_update(media) {
            Ok(Some(upd8)) => {
                let upd8_str = upd8.to_string();
                println!("{}", upd8_str);
            }
            Ok(None) => println!("No new update."),
            Err(err) => eprintln!("Error: {}", err),
        }
    }

    Ok(())
}
