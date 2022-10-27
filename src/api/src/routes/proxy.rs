use std::sync::Arc;

use db::DB;
use rest_client::RestClient;
use warp::Filter;

use crate::{
    filters::{with_db, with_rest_client},
    handlers,
};

pub fn proxy_routes(
    db: Arc<DB>,
    client: Arc<RestClient>,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    proxy_stream(db.clone(), client.clone()).or(proxy_attribute_url(db, client))
}

/// GET /stream/{id}
fn proxy_stream(
    db: Arc<DB>,
    client: Arc<RestClient>,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path!("stream" / u64)
        .and(warp::get())
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
