use std::{convert::Infallible, sync::Arc};

use db::DB;
use rest_client::RestClient;
use serde::de::DeserializeOwned;
use warp::Filter;

pub fn with_db(db: Arc<DB>) -> impl Filter<Extract = (Arc<DB>,), Error = Infallible> + Clone {
    warp::any().map(move || db.clone())
}

pub fn with_rest_client(
    client: Arc<RestClient>,
) -> impl Filter<Extract = (Arc<RestClient>,), Error = Infallible> + Clone {
    warp::any().map(move || client.clone())
}

pub fn json_body<T>() -> impl Filter<Extract = (T,), Error = warp::Rejection> + Clone
where
    T: DeserializeOwned + Send,
{
    warp::body::content_length_limit(1024 * 16).and(warp::body::json())
}
