pub mod xtream;

use std::{convert::Infallible, sync::Arc};

use db::DB;
use rest_client::RestClient;
use serde::de::DeserializeOwned;
use warp::{any, body, Filter};

use crate::models::{
    xtream::{Credentials, XtreamConfig},
    ApiConfiguration,
};

pub fn with_db(db: Arc<DB>) -> impl Filter<Extract = (Arc<DB>,), Error = Infallible> + Clone {
    any().map(move || db.clone())
}

pub fn with_rest_client(
    client: Arc<RestClient>,
) -> impl Filter<Extract = (Arc<RestClient>,), Error = Infallible> + Clone {
    any().map(move || client.clone())
}

pub fn json_body<T>() -> impl Filter<Extract = (T,), Error = warp::Rejection> + Clone
where
    T: DeserializeOwned + Send,
{
    body::content_length_limit(1024 * 16).and(warp::body::json())
}

pub fn with_group_excludes(
    group_excludes: Vec<String>,
) -> impl Filter<Extract = (Vec<String>,), Error = Infallible> + Clone {
    any().map(move || group_excludes.clone())
}

pub fn with_xtream_base_url(
    base_url: String,
) -> impl Filter<Extract = (String,), Error = Infallible> + Clone {
    any().map(move || base_url.clone())
}

pub fn with_config(
    config: ApiConfiguration,
) -> impl Filter<Extract = (ApiConfiguration,), Error = Infallible> + Clone {
    any().map(move || config.clone())
}

pub fn with_xtream_config(
    xtream_config: XtreamConfig,
) -> impl Filter<Extract = (XtreamConfig,), Error = Infallible> + Clone {
    any().map(move || xtream_config.clone())
}

pub fn with_credentials(
    credentials: Credentials,
) -> impl Filter<Extract = (Credentials,), Error = Infallible> + Clone {
    any().map(move || credentials.clone())
}
