use serde::{Deserialize, Serialize};
use sqlx::{Error, FromRow};

use crate::{Connection, CRUD};

#[derive(Debug, Clone, Deserialize)]
pub struct ProviderRequest {
    pub name: String,
    pub source: String,
    pub groups: String,
    pub channels: String,
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
pub struct Provider {}

#[async_trait::async_trait]
impl CRUD<ProviderModel, ProviderRequest> for Provider {
    async fn get(&self, tx: &mut Connection, id: u64) -> Result<ProviderModel, Error> {
        let res = sqlx::query_as!(ProviderModel, "select * from provider where id = ?", id)
            .fetch_one(tx)
            .await;

        res
    }

    async fn insert(&self, tx: &mut Connection, provider: ProviderRequest) -> Result<u64, Error> {
        let res = sqlx::query_as!(
            ProviderModel,
            r#"insert into provider (name, source, groups, channels) values (?, ?, ?, ?)"#,
            provider.name,
            provider.source,
            provider.groups,
            provider.channels
        )
        .execute(tx)
        .await?
        .last_insert_id();

        Ok(res)
    }

    async fn delete(&self, tx: &mut Connection, id: u64) -> Result<u64, Error> {
        let res = sqlx::query_as!(u64, r#"delete from provider where id = ?"#, id)
            .execute(tx)
            .await?
            .rows_affected();

        Ok(res)
    }
}
