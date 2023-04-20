use std::sync::Arc;

use anyhow::{Context, Error};
use db::{CRUD, DB};
use reqwest::{Method, Url};
use rest_client::RestClient;
use warp::{
    http::HeaderMap,
    hyper::{Body, Response},
};

use crate::{
    models::Path,
    utils::{response::ResponseUtil, url::UrlUtil},
};

#[derive(Clone)]
pub struct ProxyService {
    response_util: ResponseUtil,
    url_util: UrlUtil,
    db: Arc<DB>,
    client: Arc<RestClient>,
}

impl ProxyService {
    pub fn new(db: Arc<DB>, client: Arc<RestClient>) -> Self {
        ProxyService {
            response_util: ResponseUtil::new(),
            url_util: UrlUtil::new(),
            db,
            client,
        }
    }

    pub async fn proxy_stream(
        &self,
        path: Path,
        headers: HeaderMap,
    ) -> Result<Response<Body>, Error> {
        let mut tx = self.db.pool.begin().await?;

        let track = self.url_util.parse_track(path.id)?;

        let extinf = self
            .db
            .extinf
            .get(&mut tx, track.id)
            .await
            .context(format!("Unable to get ext entry with ID: {}", track.id))?;

        let url = Url::parse(&extinf.url)?;

        let res = self.client.request(Method::GET, url, headers).await?;

        let builder = self.response_util.compose_base_response(&res).await?;

        if self.url_util.is_hls_stream(extinf.url) {
            self.url_util
                .persist_final_response_url(res.url(), self.db.clone())
                .await?;
        }

        let res = self
            .response_util
            .compose_proxy_stream_response(res, builder)
            .await
            .context("error proxying stream")?;

        tx.commit().await?;

        return Ok(res);
    }

    pub async fn proxy_hls(&self, path: Path, headers: HeaderMap) -> Result<Response<Body>, Error> {
        let mut tx = self.db.pool.begin().await?;

        let host = self.db.hls_url.get_latest(&mut tx).await?;

        tx.commit().await?;

        let url = Url::parse(&format!(
            "{}/hls/{}/{}",
            host.url,
            path.segment1.unwrap(),
            path.id
        ))?;

        let res = self.client.request(Method::GET, url, headers).await?;

        let builder = self.response_util.compose_base_response(&res).await?;

        let res = self
            .response_util
            .compose_proxy_stream_response(res, builder)
            .await
            .context("error proxying stream")?;

        return Ok(res);
    }
}
