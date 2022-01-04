use std::{convert::Infallible, sync::Arc};

use api::handlers::root_handler;
use db::{CRUD, DB};
use rest_client::RestClient;
use url::Url;
use warp::{serve, Filter};

#[tokio::main]
async fn main() {
    let pool = db::connect().await;
    db::handle_migrations(&pool).await;

    let db = db::init_db(pool).await;

    let client = RestClient::new();

    let url = db.provider.get(1).await.expect("provider entity").source;
    let url = Url::parse(&"https://jsonplaceholder.typicode.com/todos/1").expect("parsed url");

    iptv::m3u::parser::parse_m3u(url).await;
    start_server(Arc::new(db), Arc::new(client)).await
}

pub async fn start_server(db: Arc<DB>, client: Arc<RestClient>) {
    let api = warp::get()
        .and(warp::path::end())
        .and(with_db(db))
        .and(with_rest_client(client))
        .and_then(root_handler);

    serve(api).run(([0, 0, 0, 0], 3001)).await;
}

pub fn with_db(db: Arc<DB>) -> impl Filter<Extract = (Arc<DB>,), Error = Infallible> + Clone {
    warp::any().map(move || db.clone())
}

pub fn with_rest_client(
    client: Arc<RestClient>,
) -> impl Filter<Extract = (Arc<RestClient>,), Error = Infallible> + Clone {
    warp::any().map(move || client.clone())
}
