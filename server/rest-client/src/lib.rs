use std::convert::TryInto;
use std::time::Duration;

use surf::{Client, Config, Url};

#[derive(Clone)]
pub struct RestClient {
    client: Client,
}

impl RestClient {
    pub fn new() -> Self {
        let client: Client = Config::new()
            .set_timeout(Some(Duration::from_secs(5)))
            .try_into()
            .expect("REST client created");

        Self { client }
    }

    pub async fn post(&self, url: &Url, json: &str) -> Result<(), surf::Error> {
        self.client.post(&url).body_json(&json)?.await?;

        Ok(())
    }
}
