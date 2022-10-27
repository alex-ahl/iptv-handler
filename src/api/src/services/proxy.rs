use std::sync::Arc;

use anyhow::{bail, Context, Error};
use rest_client::RestClient;
use url::Url;
use warp::{
    hyper::{Body, Response},
    Reply,
};

use db::{CRUD, DB};

#[derive(Clone)]
pub struct ProxyService {
    db: Option<Arc<DB>>,
    client: Option<Arc<RestClient>>,
}

impl ProxyService {
    pub fn new() -> Self {
        ProxyService {
            db: None,
            client: None,
        }
    }

    pub fn initialize(&mut self, db: Arc<DB>, client: Arc<RestClient>) {
        self.db = Some(db);
        self.client = Some(client);
    }

    pub async fn proxy_stream(&self, id: u64) -> Result<Response<Body>, Error> {
        if let Some(ref db) = self.db {
            let mut tx = db.pool.begin().await?;

            let extinf = db
                .extinf
                .get(&mut tx, id)
                .await
                .context(format!("Unable to get ext entry with ID: {}", id))?;

            let url = Url::parse(&extinf.url)?;

            if let Some(ref client) = self.client {
                let res = client
                    .get_bytes_stream(&url)
                    .await
                    .context(format!("Byte stream error for {}", url))?;

                let body = Body::wrap_stream(res);

                let response = Response::builder().status(200).body(body)?;

                return Ok(response);
            } else {
                bail!("Unable to initialize REST client");
            }
        } else {
            bail!("Unable to initialize db");
        }
    }

    pub async fn proxy_attribute(&self, id: u64) -> Result<Response<Body>, Error> {
        if let Some(ref db) = self.db {
            let mut tx = db.pool.begin().await?;

            let attr = db
                .attribute
                .get(&mut tx, id)
                .await
                .context(format!("Unable to get attribute entry with ID: {}", id))?;

            let url = Url::parse(&attr.value)?;

            if let Some(ref client) = self.client {
                let res = client.get_bytes(&url).await?;

                let response = Response::builder().status(200).body(res).into_response();

                return Ok(response);
            } else {
                bail!("Unable to initialize REST client");
            }
        } else {
            bail!("Unable to initialize db");
        }
    }
}
