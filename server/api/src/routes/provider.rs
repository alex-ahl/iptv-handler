use std::sync::Arc;

use crate::filters::json_body;
// use crate::DbService;
use crate::{filters::with_db, handlers};
use db::DB;
use warp::Filter;

/// All provider routes
pub fn provider_routes(
    db: Arc<DB>,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    get_provider(db.clone())
        .or(update_provider(db.clone()))
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

/// GET /providers/{guid}
fn get_provider(
    db: Arc<DB>,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path!("provider" / String)
        .and(warp::get())
        .and(with_db(db))
        .and_then(handlers::provider::get_provider)
}

/// PUT /providers/{guid}
fn update_provider(
    db: Arc<DB>,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path!("provider" / String)
        .and(warp::put())
        .and(json_body())
        .and(with_db(db))
        .and_then(handlers::provider::update_provider)
}

/// DELETE /providers/{guid}
fn delete_provider(
    db: Arc<DB>,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path!("provider" / String)
        .and(warp::delete())
        .and(with_db(db))
        .and_then(handlers::provider::delete_provider)
}
