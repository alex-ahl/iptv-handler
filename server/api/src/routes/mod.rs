use db::DB;
use rest_client::RestClient;
use std::sync::Arc;
use warp::Filter;

pub mod provider;
pub mod root;

pub fn get_routes(
    db: Arc<DB>,
    client: Arc<RestClient>,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    root::root_routes().or(provider::provider_routes(db, client))
}
