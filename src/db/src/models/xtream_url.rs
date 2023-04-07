use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use sqlx::{query_as, Error, FromRow};

use crate::{Connection, CRUD};

#[derive(Debug, Clone)]
pub struct XtreamUrlRequest {
    pub url: String,
    pub m3u_id: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct XtreamUrlModel {
    pub id: u64,
    pub url: String,

    #[serde(skip)]
    pub m3u_id: Option<u64>,
}

#[derive(Debug, Clone)]
pub struct XtreamUrl {}

impl XtreamUrl {
    pub async fn delete_by_m3u_id(
        &self,
        tx: &mut Connection,
        provider_id: u64,
    ) -> Result<u64, Error> {
        let res = query_as!(
            u64,
            "delete xtream_url from xtream_url where m3u_id = ?",
            provider_id
        )
        .execute(tx)
        .await?
        .rows_affected();

        Ok(res)
    }

    pub async fn get_latest_m3u_id(&self, tx: &mut Connection) -> Result<Option<u64>, Error> {
        let res: (Option<u64>,) = query_as("select max(m3u_id) from xtream_url")
            .fetch_one(tx)
            .await?;

        Ok(res.0)
    }
}

#[async_trait]
impl CRUD<XtreamUrlModel, XtreamUrlRequest> for XtreamUrl {
    async fn get(&self, tx: &mut Connection, id: u64) -> Result<XtreamUrlModel, Error> {
        let res = query_as!(
            XtreamUrlModel,
            "select id, url, m3u_id from xtream_url where id = ?",
            id
        )
        .fetch_one(tx)
        .await;

        res
    }

    async fn insert(
        &self,
        tx: &mut Connection,
        xtream_url_request: XtreamUrlRequest,
    ) -> Result<u64, Error> {
        let res = query_as!(
            XtreamUrlModel,
            r#"insert into xtream_url (url, m3u_id) values (?, ?)"#,
            xtream_url_request.url,
            xtream_url_request.m3u_id,
        )
        .execute(tx)
        .await?
        .last_insert_id();

        Ok(res)
    }

    async fn delete(&self, tx: &mut Connection, id: u64) -> Result<u64, Error> {
        let res = query_as!(u64, r#"delete from xtream_url where id = ?"#, id)
            .execute(tx)
            .await?
            .rows_affected();

        Ok(res)
    }
}
