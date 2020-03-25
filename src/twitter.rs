use super::*;
use anyhow::{anyhow, ensure, Result};
use egg_mode::tweet::Tweet;

impl IntoUpdate for Tweet {
    fn into_update(&self, media: &Media) -> Result<Update> {
        let user = self.user.clone().ok_or_else(|| anyhow!("Missing user!"))?;
        let link = format!(
            "https://twitter.com/{}/status/{}",
            user.screen_name, self.id
        );
        let title = self
            .text
            .split("https://t.co")
            .next()
            .unwrap_or("")
            .to_string();
        let id = self.id;
        Ok(Update {
            id,
            title,
            link,
            media: media.clone(),
            show_id: false,
        })
    }
}

pub struct TwitterFeed(pub Vec<Tweet>);

impl Feed for TwitterFeed {
    type Item = Tweet;

    fn updates(&self) -> &[Self::Item] {
        &self.0
    }

    #[tokio::main]
    async fn fetch(media: &Media) -> Result<Self> {
        ensure!(media == &Media::PQTwitter, "Media isn't on Twitter!");

        let key = dotenv::var("TWITTER_KEY")?;
        let secret = dotenv::var("TWITTER_SECRET")?;
        let keypair = egg_mode::KeyPair::new(key, secret);
        let token = egg_mode::bearer_token(&keypair).await?;
        let query =
            egg_mode::tweet::user_timeline("HSPesterquest", false, true, &token).with_page_size(50);
        let tweets = query.call(None, None).await?.response;

        Ok(Self(
            tweets
                .into_iter()
                .filter_map(|x| {
                    x.retweeted_status
                        .filter(|x| {
                            x.user
                                .as_ref()
                                .map(|x| x.screen_name == "homestuck")
                                .unwrap_or(false)
                                && !x.entities.urls.iter().any(|x| {
                                    x.expanded_url
                                        .as_ref()
                                        .unwrap_or(&x.display_url)
                                        .contains("steampowered")
                                })
                        })
                        .map(|x| *x)
                })
                .collect(),
        ))
    }
}
