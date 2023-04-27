use std::sync::Arc;

use db::DB;
use warp::{get, path, post, Filter, Rejection, Reply};

use crate::{
    filters::{json_body, with_db, with_output},
    handlers,
};

pub fn m3u_routes(db: Arc<DB>) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    get_latest_m3u_file()
        .or(get_m3u_from_disc())
        .or(get_m3u_file_exist())
        .or(create_m3u_file(db))
}

/// GET /m3u
fn get_latest_m3u_file() -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    path!("m3u")
        .and(get())
        .and(with_output())
        .and_then(handlers::m3u::get_latest_m3u_file)
}

fn get_m3u_from_disc() -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    path!("m3u" / String)
        .and(get())
        .and_then(|file_name: String| handlers::m3u::serve_file_by_file_name(file_name))
}

fn get_m3u_file_exist() -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    path!("m3u-exist")
        .and(get())
        .and(with_output())
        .and_then(handlers::m3u::m3u_file_exist)
}

/// GET /m3u/create
fn create_m3u_file(db: Arc<DB>) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    path!("m3u" / "create")
        .and(post())
        .and(json_body())
        .and(with_db(db))
        .and_then(handlers::m3u::create_m3u)
}
