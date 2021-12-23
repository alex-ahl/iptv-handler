pub mod models;

use models::Provider;
use sqlx::{migrate, Error, MySql, MySqlPool, Pool};
use std::fmt::Debug;
use std::{env, sync::Arc};

pub type ConnectionPool = Pool<MySql>;

pub async fn connect() -> ConnectionPool {
    let connection_string = env::var("DATABASE_URL").unwrap();

    MySqlPool::connect(&connection_string)
        .await
        .expect("database connection")
}

pub async fn handle_migrations(pool: &ConnectionPool) {
    migrate!("./migrations")
        .run(pool)
        .await
        .expect("database migrations");
}

#[async_trait::async_trait]
pub trait CRUD<T>: Send + Sync + Debug {
    async fn get(&self, id: u64) -> Result<T, Error>;
}

pub struct DB {
    pub provider: Arc<Provider>,
}

pub async fn init_db(pool: ConnectionPool) -> DB {
    DB {
        provider: Arc::new(Provider::new(pool.clone())),
    }
}
