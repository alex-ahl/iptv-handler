mod api;
mod db;
mod parser;
mod server;
pub mod provider;

#[tokio::main]
async fn main() {
    db::connect().await;
    parser::parser().await;
    server::start_server().await;
}
