mod api;
mod parser;
mod server;
use sqlx;

#[tokio::main]
async fn main() {
    dotenv::dotenv().expect("Failed to read .env file");
    parser::parser().await;
    server::start_server().await;

    let pool = sqlx::MySqlPool::new("mysql://db:db@127.0.0.1/iptvproxy").await;
    println!("{:?}", pool)
}
