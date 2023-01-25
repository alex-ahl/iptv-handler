use std::sync::Arc;

use db::DB;
use rest_client::RestClient;
use warp::Filter;

use crate::{
    filters::{with_config, with_db, with_rest_client},
    handlers,
    models::{ApiConfiguration, Path},
};

pub fn proxy_routes(
    config: ApiConfiguration,
    db: Arc<DB>,
    client: Arc<RestClient>,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    proxy_stream(config, db.clone(), client.clone()).or(proxy_attribute_url(db, client))
}

/// GET /stream/{id}
fn proxy_stream(
    config: ApiConfiguration,
    db: Arc<DB>,
    client: Arc<RestClient>,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path!("stream" / u64)
        .and(warp::get())
        .map(|id: u64| Path {
            segment1: None,
            segment2: None,
            segment3: None,
            id: id.to_string(),
        })
        .and(with_config(config))
        .and(with_db(db))
        .and(with_rest_client(client))
        .and_then(handlers::proxy::proxy_stream)
}

/// GET /attr/{id}
fn proxy_attribute_url(
    db: Arc<DB>,
    client: Arc<RestClient>,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path!("attr" / u64)
        .and(warp::get())
        .and(with_db(db))
        .and(with_rest_client(client))
        .and_then(handlers::proxy::proxy_attr)
}
