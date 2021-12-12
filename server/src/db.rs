use std::env;

use sqlx::{migrate, MySqlPool};

pub async fn connect() {
    let connection_string = env::var("CONNECTION_STRING").unwrap();

    let pool = MySqlPool::connect(&connection_string)
        .await
        .expect("Failed to connect to database");

    migrate!("./migrations")
        .run(&pool)
        .await
        .expect("DB migrations failed");
}
