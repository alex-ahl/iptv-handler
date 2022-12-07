use chrono::{NaiveDateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{query_as, Error, FromRow};

use crate::{Connection, CRUD};

#[derive(Debug, Clone, Deserialize)]
pub struct ProviderRequest {
    pub name: Option<String>,
    pub source: String,
    pub groups: Option<u32>,
    pub channels: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow, Default)]

pub struct ProviderModel {
    pub id: u64,
    pub name: Option<String>,
    pub source: String,
    groups: Option<u32>,
    channels: Option<u32>,
    pub created_at: Option<NaiveDateTime>,
    modified_at: Option<NaiveDateTime>,
}

#[derive(Debug, Clone)]
pub struct Provider {}

impl Provider {
    pub async fn get_all(&self, tx: &mut Connection) -> Result<Vec<ProviderModel>, Error> {
        let res = query_as!(
            ProviderModel,
            "select id, name, source, groups, channels, created_at, modified_at from provider",
        )
        .fetch_all(tx)
        .await;

        res
    }

    pub async fn get_by_url(&self, tx: &mut Connection, url: &str) -> Result<ProviderModel, Error> {
        let res = query_as!(
            ProviderModel,
            "select id, name, source, groups, channels, created_at, modified_at from provider where source = ?",
            url
        )
        .fetch_one(tx)
        .await;

        res
    }

    pub async fn exists(&self, tx: &mut Connection, url: &str) -> Result<bool, Error> {
        let res = query_as!(
            ProviderModel,
            "select * from provider where source = ?",
            url
        )
        .fetch_one(tx)
        .await
        .is_ok();

        Ok(res)
    }
}

#[async_trait::async_trait]
impl CRUD<ProviderModel, ProviderRequest> for Provider {
    async fn get(&self, tx: &mut Connection, id: u64) -> Result<ProviderModel, Error> {
        let res = query_as!(
            ProviderModel,
            "select id, name, source, groups, channels, created_at, modified_at from provider where id = ?",
            id
        )
        .fetch_one(tx)
        .await;
        res
    }

    async fn insert(&self, tx: &mut Connection, provider: ProviderRequest) -> Result<u64, Error> {
        let res = query_as!(
            ProviderModel,
            r#"insert into provider (name, source, groups, channels, created_at, modified_at) values (?, ?, ?, ?, ?, ?)"#,
            provider.name,
            provider.source,
            provider.groups,
            provider.channels,
            Utc::now(),
            Utc::now(),
        )
        .execute(tx)
        .await?
        .last_insert_id();

        Ok(res)
    }

    async fn delete(&self, tx: &mut Connection, id: u64) -> Result<u64, Error> {
        let res = query_as!(u64, r#"delete from provider where id = ?"#, id)
            .execute(tx)
            .await?
            .rows_affected();

        Ok(res)
    }
}
