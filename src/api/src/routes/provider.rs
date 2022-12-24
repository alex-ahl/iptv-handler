use std::sync::Arc;

use crate::filters::{json_body, with_group_excludes};
use crate::models::ApiConfiguration;
use crate::{filters::with_db, handlers};
use db::DB;
use warp::{delete, get, path, post, Filter, Rejection, Reply};

/// All provider routes
pub fn provider_routes(
    config: ApiConfiguration,
    db: Arc<DB>,
) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    refresh_all_providers(&config.group_excludes, db.clone())
        .or(get_provider(db.clone()))
        .or(delete_provider(db.clone()))
        .or(create_provider(&config.group_excludes, db.clone()))
}

/// POST /provider
fn create_provider(
    group_excludes: &Vec<String>,
    db: Arc<DB>,
) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    path("provider")
        .and(post())
        .and(json_body())
        .and(with_group_excludes(group_excludes.to_owned()))
        .and(with_db(db))
        .and_then(handlers::provider::create_provider)
}

/// GET /provider/{u64}
fn get_provider(db: Arc<DB>) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    path!("provider" / u64)
        .and(get())
        .and(with_db(db))
        .and_then(handlers::provider::get_provider)
}

/// GET /provider/refresh
fn refresh_all_providers(
    group_excludes: &Vec<String>,
    db: Arc<DB>,
) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    path!("provider" / "refresh")
        .and(get())
        .and(with_group_excludes(group_excludes.to_owned()))
        .and(with_db(db))
        .and_then(handlers::provider::refresh_providers)
}

/// DELETE /providers/{u64}
fn delete_provider(db: Arc<DB>) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    path!("provider" / u64)
        .and(delete())
        .and(with_db(db))
        .and_then(handlers::provider::delete_provider)
}
