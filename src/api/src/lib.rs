use std::{convert::Infallible, sync::Arc};

use db::DB;
use models::ApiConfiguration;
use rest_client::RestClient;
use routes::get_routes;
use warp::Filter;

pub mod filters;
pub mod handlers;
pub mod models;
pub mod routes;
mod services;
pub mod utils;

pub fn init_api(
    config: ApiConfiguration,
    db: Arc<DB>,
    client: Arc<RestClient>,
) -> impl Filter<Extract = impl warp::Reply, Error = Infallible> + Clone {
    get_routes(config, db, client)
}
