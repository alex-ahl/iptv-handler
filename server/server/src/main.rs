use api::filters;
use warp::{log, serve, Filter};

#[tokio::main]
async fn main() {
    db::connect().await;
    iptv::parser().await;
    start_server().await
}

pub async fn start_server() {
    let api = filters::m3us();
    let routes = api.with(log("m3us"));

    serve(routes).run(([127, 0, 0, 1], 3030)).await;
}
