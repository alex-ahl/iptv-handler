use std::env;
// use url::{Url};
use sqlx::{migrate, MySqlPool};

pub async fn connect() {
    let connection_string = env::var("DATABASE_URL").unwrap();
    let pool = MySqlPool::connect(&connection_string)
        .await
        .expect("Failed to connect to database");

    migrate!("./migrations")
        .run(&pool)
        .await
        .expect("DB migrations faile");

    let provider = sqlx::query_as!(Provider, "select * from Provider")
        .fetch_one(&pool)
        .await;

    println!("{:?}", provider)
}

#[derive(Debug)]
struct Provider {
    id: u64,
    name: String,
    source: String,
    groups: String,
    channels: String,
}
