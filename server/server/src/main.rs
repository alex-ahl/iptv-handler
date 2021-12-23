use api::filters;
use db::CRUD;
use warp::{log, serve, Filter};

#[tokio::main]
async fn main() {
    let pool = db::connect().await;
    db::handle_migrations(&pool).await;

    let db = db::init_db(pool).await;
    iptv::parser().await;
    start_server().await
}

pub async fn start_server() {
    let api = filters::m3us();
    let routes = api.with(log("m3us"));

    serve(routes).run(([127, 0, 0, 1], 3030)).await;
}
