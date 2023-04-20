use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use sqlx::{query, query_as, Error, FromRow};

use crate::{Connection, CRUD};

#[derive(Debug, Clone)]
pub struct HlsUrlRequest {
    pub url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct HlsUrlModel {
    pub id: u64,
    pub url: String,
}

#[derive(Debug, Clone)]
pub struct HlsUrl {}

impl HlsUrl {
    pub async fn get_latest(&self, tx: &mut Connection) -> Result<HlsUrlModel, Error> {
        let res = query_as!(HlsUrlModel, "select id, url from hls_url",)
            .fetch_one(tx)
            .await;

        res
    }

    pub async fn truncate(&self, tx: &mut Connection) -> Result<u64, Error> {
        let res = query!("truncate table hls_url")
            .execute(tx)
            .await?
            .rows_affected();

        Ok(res)
    }
}

#[async_trait]
impl CRUD<HlsUrlModel, HlsUrlRequest> for HlsUrl {
    async fn get(&self, tx: &mut Connection, id: u64) -> Result<HlsUrlModel, Error> {
        let res = query_as!(HlsUrlModel, "select id, url from hls_url where id = ?", id)
            .fetch_one(tx)
            .await;

        res
    }

    async fn insert(
        &self,
        tx: &mut Connection,
        hls_url_request: HlsUrlRequest,
    ) -> Result<u64, Error> {
        let res = query_as!(
            HlsUrlModel,
            r#"insert into hls_url (url) values (?)"#,
            hls_url_request.url,
        )
        .execute(tx)
        .await?
        .last_insert_id();

        Ok(res)
    }

    async fn delete(&self, tx: &mut Connection, id: u64) -> Result<u64, Error> {
        let res = query_as!(u64, r#"delete from hls_url where id = ?"#, id)
            .execute(tx)
            .await?
            .rows_affected();

        Ok(res)
    }
}
