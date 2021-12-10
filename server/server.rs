use warp::{log, serve, Filter};

use crate::api;

pub async fn start_server() {
    let api = api::filters::m3us();
    let routes = api.with(log("m3us"));

    serve(routes).run(([127, 0, 0, 1], 3030)).await;
}
