mod api;
mod db;
mod parser;
mod server;

#[tokio::main]
async fn main() {
    db::connect().await;
    parser::parser().await;
    server::start_server().await;
}
