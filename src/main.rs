use anyhow::Result;
use upd8r::*;

fn main() -> Result<()> {
    for item in hs2_feed()?.items() {
        let update = Update::from_item(Media::Homestuck2, &item)?;
        if update.latest()? {
            println!("{}", update);
        }
    }

    Ok(())
}
