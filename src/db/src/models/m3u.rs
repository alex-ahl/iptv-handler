use chrono::{NaiveDateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{query_as, Error, FromRow};

use crate::{Connection, CRUD};

#[derive(Debug, Clone)]
pub struct M3uRequest {
    pub provider_id: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]

pub struct M3uModel {
    id: u64,
    created_at: Option<NaiveDateTime>,
    modified_at: Option<NaiveDateTime>,

    #[serde(skip)]
    pub provider_id: Option<u64>,
}

#[derive(Debug, Clone)]
pub struct M3u {}

impl M3u {
    pub async fn delete_by_provider_id(
        &self,
        tx: &mut Connection,
        provider_id: u64,
    ) -> Result<u64, Error> {
        let res = sqlx::query_as!(
            u64,
            "delete m3u from m3u where provider_id = ?",
            provider_id
        )
        .execute(tx)
        .await?
        .rows_affected();

        Ok(res)
    }
}

#[async_trait::async_trait]
impl CRUD<M3uModel, M3uRequest> for M3u {
    async fn get(&self, tx: &mut Connection, id: u64) -> Result<M3uModel, Error> {
        let res = query_as!(M3uModel, "select * from m3u where id = ?", id)
            .fetch_one(tx)
            .await;

        res
    }

    async fn insert(&self, tx: &mut Connection, m3u: M3uRequest) -> Result<u64, Error> {
        let res = query_as!(
            M3uModel,
            r#"insert into m3u (provider_id, created_at, modified_at) values (?, ?, ?)"#,
            m3u.provider_id,
            Utc::now(),
            Utc::now(),
        )
        .execute(tx)
        .await?
        .last_insert_id();

        Ok(res)
    }

    async fn delete(&self, tx: &mut Connection, id: u64) -> Result<u64, Error> {
        let res = query_as!(u64, r#"delete from m3u where id = ?"#, id)
            .execute(tx)
            .await?
            .rows_affected();

        Ok(res)
    }
}
