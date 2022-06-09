use warp::hyper::StatusCode;

pub async fn root_handler() -> Result<impl warp::Reply, warp::Rejection> {
    Ok(warp::reply::with_status("Not found", StatusCode::NOT_FOUND))
}
