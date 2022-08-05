use db::DB;
use std::sync::Arc;
use warp::Filter;

use self::m3u::m3u_routes;

pub mod m3u;
pub mod provider;
pub mod root;

pub fn get_routes(
    db: Arc<DB>,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    root::root_routes()
        .or(provider::provider_routes(db))
        .or(m3u_routes())
}
