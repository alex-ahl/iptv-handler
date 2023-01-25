use db::DB;
use rest_client::RestClient;
use std::{convert::Infallible, sync::Arc};
use warp::Filter;

use crate::models::ApiConfiguration;

use self::{
    m3u::m3u_routes, provider::provider_routes, proxy::proxy_routes, root::root_routes,
    xtream::xtream_routes,
};

pub mod m3u;
pub mod provider;
pub mod proxy;
pub mod root;
pub mod xtream;

pub fn get_routes(
    config: ApiConfiguration,
    db: Arc<DB>,
    client: Arc<RestClient>,
) -> impl Filter<Extract = impl warp::Reply, Error = Infallible> + Clone {
    root_routes()
        .or(provider_routes(config.clone(), db.clone(), client.clone()))
        .or(m3u_routes(db.clone()))
        .or(proxy_routes(config.clone(), db.clone(), client.clone()))
        .or(xtream_routes(config, client, db))
}
