use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use sqlx::{query, query_as, Error, FromRow};

use crate::{Connection, CRUD};

#[derive(Debug, Clone)]
pub struct XmltvUrlRequest {
    pub url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct XmltvUrlModel {
    pub id: u64,
    pub url: String,
}

#[derive(Debug, Clone)]
pub struct XmltvUrl {}

impl XmltvUrl {
    pub async fn get_latest(&self, tx: &mut Connection) -> Result<XmltvUrlModel, Error> {
        let res = query_as!(XmltvUrlModel, "select id, url from xmltv_url",)
            .fetch_one(tx)
            .await;

        res
    }

    pub async fn truncate(&self, tx: &mut Connection) -> Result<u64, Error> {
        let res = query!("truncate table xmltv_url")
            .execute(tx)
            .await?
            .rows_affected();

        Ok(res)
    }
}

#[async_trait]
impl CRUD<XmltvUrlModel, XmltvUrlRequest> for XmltvUrl {
    async fn get(&self, tx: &mut Connection, id: u64) -> Result<XmltvUrlModel, Error> {
        let res = query_as!(
            XmltvUrlModel,
            "select id, url from xmltv_url where id = ?",
            id
        )
        .fetch_one(tx)
        .await;

        res
    }

    async fn insert(
        &self,
        tx: &mut Connection,
        xmltv_url_request: XmltvUrlRequest,
    ) -> Result<u64, Error> {
        let res = query_as!(
            XmltvUrlModel,
            r#"insert into xmltv_url (url) values (?)"#,
            xmltv_url_request.url,
        )
        .execute(tx)
        .await?
        .last_insert_id();

        Ok(res)
    }

    async fn delete(&self, tx: &mut Connection, id: u64) -> Result<u64, Error> {
        let res = query_as!(u64, r#"delete from xmltv_url where id = ?"#, id)
            .execute(tx)
            .await?
            .rows_affected();

        Ok(res)
    }
}
