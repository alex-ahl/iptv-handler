use std::sync::Arc;

use anyhow::{Context, Error};
use db::{CRUD, DB};
use reqwest::Url;
use rest_client::RestClient;
use serde::de::DeserializeOwned;

use crate::models::ResponseData;
use warp::{
    hyper::{Body, Response},
    Reply,
};

use super::response::ResponseUtil;

#[derive(Clone)]
pub struct ProxyUtil {
    response_util: ResponseUtil,
    db: Arc<DB>,
    client: Arc<RestClient>,
}

impl ProxyUtil {
    pub fn new(response_util: ResponseUtil, db: Arc<DB>, client: Arc<RestClient>) -> Self {
        ProxyUtil {
            response_util,
            db,
            client,
        }
    }

    pub async fn proxy_request_json<T>(&self, url: &Url) -> Result<ResponseData<T>, Error>
    where
        T: DeserializeOwned + Send,
    {
        let res = self.client.get(&url).await.context("error on proxy")?;
        let headers = res.headers().clone();
        let status_code = res.status().clone();

        let data = res.json::<T>().await.context("deserialize json body")?;

        Ok(ResponseData {
            data,
            headers,
            status_code,
        })
    }

    pub async fn proxy_request_bytes(&self, url: &Url) -> Result<Response<Body>, Error> {
        let res = self.client.get(&url).await.context("error on proxy")?;
        let headers = res.headers();
        let status = res.status();

        let mut builder = Response::builder().status(status);

        for (key, val) in headers.iter() {
            builder = builder.header(key, val);
        }

        let bytes = res.bytes().await.context("error getting proxy bytes")?;

        let response = builder.body(bytes).into_response();

        Ok(response)
    }

    pub async fn proxy_attribute(&self, id: u64) -> Result<Response<Body>, Error> {
        let mut tx = self.db.pool.begin().await?;

        let attr = self
            .db
            .attribute
            .get(&mut tx, id)
            .await
            .context(format!("Unable to get attribute entry with ID: {}", id))?;

        let url = Url::parse(&attr.value)?;

        let res = self.client.get(&url).await.context("error on proxy")?;

        let builder = self.response_util.compose_base_response(&res).await?;

        let res = self
            .response_util
            .compose_byte_response(res, builder)
            .await
            .context("error proxying attribute")?;

        return Ok(res);
    }
}
