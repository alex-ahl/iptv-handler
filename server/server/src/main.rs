mod app;
pub mod environment;
mod logger;

use std::sync::Arc;

use api::init_api;
use app::init_app;
use db::{handle_migrations, init_db};
use environment::init_env;
use warp::{serve, Filter};

use crate::{environment::Configuration, logger::init_logger};

#[tokio::main]
async fn main() {
    init_logger();
    let env: Configuration = init_env();

    let pool = db::connect(env.database_url).await;
    handle_migrations(&pool).await;

    let db = init_db(pool).await;
    let db = Arc::new(db);

    let api = init_api(db.clone()).with(warp::log("warp-server"));

    if env.backend_mode_only {
        init_app(env.m3u, db).await;
    }

    serve(api).run(([0, 0, 0, 0], 3001)).await;
}
