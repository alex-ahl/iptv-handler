use crate::{ConnectionPool, CRUD};
use serde::{Deserialize, Serialize};
use sqlx::Error;
use std::fmt::Debug;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderModel {
    id: u64,
    name: String,
    source: String,
    groups: String,
    channels: String,
}

#[derive(Debug, Clone)]
pub struct Provider {
    db: ConnectionPool,
}

impl Provider {
    pub fn new(db: ConnectionPool) -> Provider {
        Provider { db }
    }
}

#[async_trait::async_trait]
impl CRUD<ProviderModel> for Provider {
    async fn get(&self, id: u64) -> Result<ProviderModel, Error> {
        let res = sqlx::query_as!(ProviderModel, "select * from Provider where id = ?", id)
            .fetch_one(&self.db)
            .await;

        res
    }
}
