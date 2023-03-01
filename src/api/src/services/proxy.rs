use std::sync::Arc;

use anyhow::{bail, Context, Error};
use reqwest::Method;
use rest_client::RestClient;
use warp::{
    http::HeaderMap,
    hyper::{Body, Response},
};

use db::{services::provider::ProviderDBService, CRUD, DB};

use crate::{
    models::{ApiConfiguration, Path},
    utils::{response::ResponseUtil, url::UrlUtil},
};

#[derive(Clone)]
pub struct ProxyService {
    response_util: ResponseUtil,
    url_util: UrlUtil,
    config: ApiConfiguration,
    db: Arc<DB>,
    client: Arc<RestClient>,
}

impl ProxyService {
    pub fn new(config: ApiConfiguration, db: Arc<DB>, client: Arc<RestClient>) -> Self {
        ProxyService {
            response_util: ResponseUtil::new(),
            url_util: UrlUtil::new(),
            config,
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

        let mut provider_db_service = ProviderDBService::new();
        provider_db_service.initialize_db(self.db.clone());

        match provider_db_service
            .get_latest_provider_entry(self.config.m3u_url.as_str())
            .await
        {
            Some(latest_provider_entry) => {
                let m3u = self
                    .db
                    .m3u
                    .get(&mut tx, latest_provider_entry.id.clone())
                    .await
                    .context(format!(
                        "Unable to get m3u entry with id: {}",
                        latest_provider_entry.id
                    ))?;

                let url = self.url_util.compose_proxy_stream_url(
                    path.clone(),
                    m3u.clone(),
                    None,
                    None,
                )?;

                let res = self.client.request(Method::GET, url, headers).await?;

                let builder = self.response_util.compose_base_response(&res).await?;

                let res = self
                    .response_util
                    .compose_proxy_stream_response(res, builder)
                    .await
                    .context("error proxying stream")?;

                return Ok(res);
            }
            None => bail!("Unable to init provider service"),
        }
    }
}
