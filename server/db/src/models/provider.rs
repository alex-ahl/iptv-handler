use serde::{Deserialize, Serialize};
use sqlx::{Error, FromRow};

use crate::{ConnectionPool, CRUD};

#[derive(Debug, Clone)]
pub struct ProviderRequest {
    name: String,
    source: String,
    groups: String,
    channels: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]

pub struct ProviderModel {
    id: u64,
    name: String,
    pub source: String,
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
impl CRUD<ProviderModel, ProviderRequest> for Provider {
    async fn get(&self, id: u64) -> Result<ProviderModel, Error> {
        let res = sqlx::query_as!(ProviderModel, "select * from provider where id = ?", id)
            .fetch_one(&self.db)
            .await;

        res
    }

    async fn insert(&self, provider: ProviderRequest) -> Result<u64, Error> {
        let res = sqlx::query_as!(
            ProviderModel,
            r#"insert into provider (name, source, groups, channels) values (?, ?, ?, ?)"#,
            provider.name,
            provider.source,
            provider.groups,
            provider.channels
        )
        .execute(&self.db)
        .await?
        .last_insert_id();

        Ok(res)
    }

    async fn delete(&self, id: u64) -> Result<u64, Error> {
        let res = sqlx::query_as!(u64, r#"delete from provider where id = ?"#, id)
            .execute(&self.db)
            .await?
            .rows_affected();

        Ok(res)
    }
}
