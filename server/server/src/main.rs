mod app;
pub mod environment;
mod logger;

use std::sync::Arc;

use api::init_api;
use app::init_app;
use db::{handle_migrations, init_db};
use environment::init_env;
use rest_client::RestClient;
use warp::{serve, Filter};

use crate::{environment::Configuration, logger::init_logger};

#[tokio::main]
async fn main() {
    init_logger();
    let environment: Configuration = init_env();

    let pool = db::connect().await;
    handle_migrations(&pool).await;

    let db = init_db(pool).await;
    let client = RestClient::new();

    let api = init_api(Arc::new(db), Arc::new(client.clone())).with(warp::log("warp-server"));

    if environment.backend_mode_only {
        init_app(client).await;
    }

    serve(api).run(([0, 0, 0, 0], 3001)).await;
}
