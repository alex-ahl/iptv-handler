pub mod models;
use models::{Attribute, ExtInf, M3u, Provider};
use sqlx::{migrate, Error, MySql, MySqlConnection, MySqlPool, Pool};
use std::fmt::Debug;
use std::{env, sync::Arc};

pub type ConnectionPool = Pool<MySql>;
pub type Connection = MySqlConnection;

pub async fn connect() -> ConnectionPool {
    let connection_string = env::var("DATABASE_URL").unwrap();

    MySqlPool::connect(&connection_string)
        .await
        .expect("creating database connection")
}

pub async fn handle_migrations(pool: &ConnectionPool) {
    migrate!("./migrations")
        .run(pool)
        .await
        .expect("running database migrations");
}

#[async_trait::async_trait]
pub trait CRUD<TReturn, TInsert>: Send + Sync + Debug {
    async fn get(&self, tx: &mut Connection, id: u64) -> Result<TReturn, Error>;
    async fn insert(&self, tx: &mut Connection, model: TInsert) -> Result<u64, Error>;
    async fn delete(&self, tx: &mut Connection, id: u64) -> Result<u64, Error>;
}

#[derive(Debug)]
pub struct DB {
    pub pool: Arc<ConnectionPool>,

    pub provider: Provider,
    pub m3u: M3u,
    pub extinf: ExtInf,
    pub attribute: Attribute,
}

pub async fn init_db(pool: ConnectionPool) -> DB {
    DB {
        pool: Arc::new(pool.clone()),

        provider: Provider {},
        attribute: Attribute {},
        m3u: M3u {},
        extinf: ExtInf {},
    }
}
