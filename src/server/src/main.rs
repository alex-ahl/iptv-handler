mod app;
pub mod environment;
mod jobs;
mod logger;
mod tools;

use std::sync::Arc;

use api::init_api;
use app::init_app;
use db::{handle_migrations, init_db};
use environment::{init_env, map_api_configuration};
use jobs::init_jobs;
use rest_client::RestClient;
use warp::serve;

use crate::{environment::Configuration, logger::init_logger};

#[tokio::main]
async fn main() {
    init_logger();
    let config: Configuration = init_env();

    let pool = db::connect(config.database_url.clone()).await;
    handle_migrations(&pool).await;

    let db = init_db(pool).await;
    let db = Arc::new(db);

    let client = Arc::new(RestClient::new());

    let api = init_api(map_api_configuration(config.clone()), db.clone(), client);

    if config.backend_mode_only {
        init_app(config.clone(), db.clone()).await;
    }

    init_jobs(config, db);

    serve(api).run(([0, 0, 0, 0], 3001)).await;
}
