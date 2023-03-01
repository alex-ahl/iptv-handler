use std::{convert::Infallible, sync::Arc};

use ::warp::http::HeaderMap;
use db::DB;
use log::error;
use rest_client::RestClient;
use warp::hyper::{Body, Response};

use crate::{
    models::{ApiConfiguration, Path},
    services::proxy::ProxyService,
    utils::{proxy::ProxyUtil, response::ResponseUtil},
};

#[derive(Clone)]
pub struct ProxyHandler {
    proxy_service: ProxyService,
    proxy_util: ProxyUtil,
}

impl ProxyHandler {
    pub fn new(config: ApiConfiguration, db: Arc<DB>, client: Arc<RestClient>) -> Self {
        ProxyHandler {
            proxy_service: ProxyService::new(config, db.clone(), client.clone()),
            proxy_util: ProxyUtil::new(ResponseUtil::new(), db, client),
        }
    }

    pub async fn proxy_stream(
        self,
        path: Path,
        headers: HeaderMap,
    ) -> Result<Response<Body>, Infallible> {
        let res = match self.proxy_service.proxy_stream(path.clone(), headers).await {
            Ok(res) => res,
            Err(err) => {
                error!("Failed to proxy stream with id {}, error: {}", path.id, err);
                Response::builder()
                    .status(500)
                    .body(Body::from(format!("Error on stream proxy {}", path.id)))
                    .unwrap_or_default()
            }
        };
        Ok(res)
    }

    pub async fn proxy_attr(self, id: u64) -> Result<Response<Body>, Infallible> {
        let res = match self.proxy_util.proxy_attribute(id).await {
            Ok(res) => res,
            Err(err) => {
                error!("Failed to proxy stream with id {}, error: {}", id, err);
                Response::builder()
                    .status(500)
                    .body(Body::from(format!("Error on attribute proxy {}", id)))
                    .unwrap_or_default()
            }
        };

        Ok(res)
    }
}
