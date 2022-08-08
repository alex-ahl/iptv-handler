use warp::Filter;

use crate::handlers;

pub fn m3u_routes() -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    get_latest_m3u_file()
        .or(get_m3u_from_disc())
        .or(get_m3u_file_exist())
}

/// GET /m3u
fn get_latest_m3u_file() -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone
{
    warp::path!("m3u")
        .and(warp::get())
        .and_then(handlers::m3u::get_latest_m3u_file)
}

fn get_m3u_from_disc() -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path!("m3u" / String)
        .and(warp::get())
        .and_then(|file_name: String| handlers::m3u::serve_file_by_file_name(file_name))
}

fn get_m3u_file_exist() -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone
{
    warp::path!("m3u-exist")
        .and(warp::get())
        .and_then(handlers::m3u::m3u_file_exist)
}
