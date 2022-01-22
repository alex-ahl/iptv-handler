use warp::Filter;

use crate::handlers::root::root_handler;

pub fn root_routes() -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::get().and(warp::path::end()).and_then(root_handler)
}
