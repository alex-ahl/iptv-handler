mod api;
mod parser;
mod server;

#[tokio::main]
async fn main() {
    dotenv::dotenv().expect("Failed to read .env file");
    parser::parser().await;
    server::start_server().await;
}
