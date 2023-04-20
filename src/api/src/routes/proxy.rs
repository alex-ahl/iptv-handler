use std::sync::Arc;

use db::DB;
use rest_client::RestClient;
use warp::{header::headers_cloned, Filter};

use crate::{filters::with_proxy_handler, handlers::proxy::ProxyHandler, models::Path};

pub fn proxy_routes(
    db: Arc<DB>,
    client: Arc<RestClient>,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    let handler = ProxyHandler::new(db, client);

    proxy_stream(handler.clone())
        .or(proxy_attribute_url(handler.clone()))
        .or(proxy_hls(handler))
}

/// GET /stream/{id}
fn proxy_stream(
    handler: ProxyHandler,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path!("stream" / u64)
        .and(warp::get())
        .map(|id: u64| Path {
            segment1: None,
            segment2: None,
            segment3: None,
            id: id.to_string(),
        })
        .and(headers_cloned())
        .and(with_proxy_handler(handler))
        .and_then(|path, headers, handler: ProxyHandler| handler.proxy_stream(path, headers))
}

/// GET /attr/{id}
fn proxy_attribute_url(
    handler: ProxyHandler,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path!("attr" / u64)
        .and(warp::get())
        .and(with_proxy_handler(handler))
        .and_then(|id, handler: ProxyHandler| handler.proxy_attr(id))
}

fn proxy_hls(
    handler: ProxyHandler,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path!("hls" / String / String)
        .and(warp::get())
        .map(|segment1: String, id: String| Path {
            segment1: Some(segment1),
            segment2: None,
            segment3: None,
            id: id.to_string(),
        })
        .and(headers_cloned())
        .and(with_proxy_handler(handler))
        .and_then(|path, headers, handler: ProxyHandler| handler.proxy_hls(path, headers))
}
