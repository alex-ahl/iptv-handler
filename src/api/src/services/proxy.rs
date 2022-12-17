use std::sync::Arc;

use anyhow::{bail, Context, Error};
use rest_client::RestClient;
use url::Url;
use warp::{
    http::response::Builder,
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
                let res = client.get(&url).await.context("error on proxy")?;
                let builder = self.compose_base_response(&res).await?;

                let res = self
                    .compose_proxy_stream_response(res, builder)
                    .await
                    .context("error proxying stream")?;

                return Ok(res);
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
                let res = client.get(&url).await.context("error on proxy")?;

                let builder = self.compose_base_response(&res).await?;

                let res = self
                    .compose_proxy_attribute_response(res, builder)
                    .await
                    .context("error proxying attribute")?;

                return Ok(res);
            } else {
                bail!("Unable to initialize REST client");
            }
        } else {
            bail!("Unable to initialize db");
        }
    }

    async fn compose_base_response(&self, res: &reqwest::Response) -> Result<Builder, Error> {
        let status = res.status();
        let headers = res.headers();

        let mut builder = Response::builder().status(status);

        for (key, val) in headers.iter() {
            builder = builder.header(key, val);
        }

        Ok(builder)
    }

    async fn compose_proxy_stream_response(
        &self,
        res: reqwest::Response,
        response_builder: Builder,
    ) -> Result<Response<Body>, Error> {
        let bytes_stream = res.bytes_stream();
        let body = Body::wrap_stream(bytes_stream);

        let response = response_builder.body(body).into_response();

        Ok(response)
    }

    async fn compose_proxy_attribute_response(
        &self,
        res: reqwest::Response,
        response_builder: Builder,
    ) -> Result<Response<Body>, Error> {
        let bytes = res.bytes().await?;

        let response = response_builder.body(bytes).into_response();

        Ok(response)
    }
}
