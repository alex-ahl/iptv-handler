mod logger;

use std::sync::Arc;

use api::init_api;
use rest_client::RestClient;
use warp::{serve, Filter};

use crate::logger::init_logger;

#[tokio::main]
async fn main() {
    init_logger();

    let pool = db::connect().await;
    db::handle_migrations(&pool).await;

    let db = db::init_db(pool).await;
    let client = RestClient::new();

    let api = init_api(Arc::new(db), Arc::new(client)).with(warp::log("warp-server"));

    serve(api).run(([0, 0, 0, 0], 3001)).await;
}
