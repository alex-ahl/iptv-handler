use std::sync::Arc;

use crate::filters::{json_body, with_config, with_rest_client};
use crate::models::ApiConfiguration;
use crate::{filters::with_db, handlers};
use db::DB;
use rest_client::RestClient;
use warp::{delete, get, path, post, Filter, Rejection, Reply};

/// All provider routes
pub fn provider_routes(
    config: ApiConfiguration,
    db: Arc<DB>,
    client: Arc<RestClient>,
) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    refresh_all_providers(config.clone(), db.clone(), client.clone())
        .or(get_provider(db.clone()))
        .or(delete_provider(db.clone()))
        .or(create_provider(config, db.clone(), client))
}

/// POST /provider
fn create_provider(
    config: ApiConfiguration,
    db: Arc<DB>,
    client: Arc<RestClient>,
) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    path("provider")
        .and(post())
        .and(json_body())
        .and(with_config(config))
        .and(with_db(db))
        .and(with_rest_client(client))
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
    config: ApiConfiguration,
    db: Arc<DB>,
    client: Arc<RestClient>,
) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    path!("provider" / "refresh")
        .and(get())
        .and(with_config(config))
        .and(with_db(db))
        .and(with_rest_client(client))
        .and_then(handlers::provider::refresh_providers)
}

/// DELETE /providers/{u64}
fn delete_provider(db: Arc<DB>) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    path!("provider" / u64)
        .and(delete())
        .and(with_db(db))
        .and_then(handlers::provider::delete_provider)
}
