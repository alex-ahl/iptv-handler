use std::sync::Arc;

use crate::filters::json_body;
use crate::{filters::with_db, handlers};
use db::DB;
use warp::{delete, get, path, post, Filter, Rejection, Reply};

/// All provider routes
pub fn provider_routes(
    db: Arc<DB>,
) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    refresh_all_providers(db.clone())
        .or(get_provider(db.clone()))
        .or(delete_provider(db.clone()))
        .or(create_provider(db.clone()))
        .or(provider_list(db.clone()))
}

/// GET /providers
fn provider_list(db: Arc<DB>) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    path("provider")
        .and(get())
        .and(with_db(db))
        .and_then(handlers::provider::list_providers)
}

/// POST /provider
fn create_provider(db: Arc<DB>) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    path("provider")
        .and(post())
        .and(json_body())
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
    db: Arc<DB>,
) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    path!("provider" / "refresh")
        .and(get())
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
