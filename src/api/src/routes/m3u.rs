use warp::Filter;

use crate::handlers;

pub fn m3u_routes() -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    get_latest_m3u_file()
}

/// GET /m3u
fn get_latest_m3u_file() -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone
{
    warp::path!("m3u")
        .and(warp::get())
        .and_then(handlers::m3u::get_latest_m3u_file)
}
