use std::{convert::Infallible, sync::Arc};

use db::DB;
use models::xtream::XtreamConfig;
use rest_client::RestClient;
use routes::get_routes;
use warp::Filter;

pub mod filters;
pub mod handlers;
pub mod models;
pub mod routes;
mod services;

pub fn init_api(
    db: Arc<DB>,
    client: Arc<RestClient>,
    xtream_config: XtreamConfig,
) -> impl Filter<Extract = impl warp::Reply, Error = Infallible> + Clone {
    get_routes(db, client, xtream_config)
}
