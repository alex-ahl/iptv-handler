use db::DB;
use rest_client::RestClient;
use std::sync::Arc;
use warp::Filter;

use self::{m3u::m3u_routes, provider::provider_routes, proxy::proxy_routes, root::root_routes};

pub mod m3u;
pub mod provider;
mod proxy;
pub mod root;

pub fn get_routes(
    db: Arc<DB>,
    client: Arc<RestClient>,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    root_routes()
        .or(provider_routes(db.clone()))
        .or(m3u_routes(db.clone()))
        .or(proxy_routes(db, client))
}
