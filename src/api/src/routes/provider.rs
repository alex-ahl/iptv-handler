use std::sync::Arc;

use crate::filters::json_body;
use crate::{filters::with_db, handlers};
use db::DB;
use warp::Filter;

/// All provider routes
pub fn provider_routes(
    db: Arc<DB>,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    get_provider(db.clone())
        .or(delete_provider(db.clone()))
        .or(create_provider(db.clone()))
        .or(provider_list(db.clone()))
}

/// GET /providers
fn provider_list(
    db: Arc<DB>,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path("provider")
        .and(warp::get())
        .and(with_db(db))
        .and_then(handlers::provider::list_providers)
}

/// POST /provider
fn create_provider(
    db: Arc<DB>,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path("provider")
        .and(warp::post())
        .and(json_body())
        .and(with_db(db))
        .and_then(handlers::provider::create_provider)
}

/// GET /provider/{u64}
fn get_provider(
    db: Arc<DB>,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path!("provider" / u64)
        .and(warp::get())
        .and(with_db(db))
        .and_then(handlers::provider::get_provider)
}

/// DELETE /providers/{u64}
fn delete_provider(
    db: Arc<DB>,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path!("provider" / u64)
        .and(warp::delete())
        .and(with_db(db))
        .and_then(handlers::provider::delete_provider)
}
