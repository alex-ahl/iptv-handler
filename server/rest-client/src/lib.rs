use std::convert::TryInto;
use std::time::Duration;

use serde::{Deserialize, Serialize};
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

    pub async fn get_json(&self, url: &Url) -> String {
        let Res { res } = self
            .client
            .get(&url)
            .recv_json()
            .await
            .expect("parsed JSON");

        res
    }

    pub async fn post(&self, url: &Url, json: &str) -> Result<(), surf::Error> {
        self.client.post(&url).body_json(&json)?.await?;

        Ok(())
    }
}

#[derive(Deserialize, Serialize)]
struct Res {
    res: String,
}
