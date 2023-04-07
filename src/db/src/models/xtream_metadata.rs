use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use sqlx::{query_as, Error, FromRow};

use crate::{Connection, CRUD};

#[derive(Debug, Clone)]
pub struct XtreamMetadataRequest {
    pub metadata: String,
    pub metadata_type: String,
    pub m3u_id: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct XtreamMetadataModel {
    pub id: u64,
    pub metadata: String,
    pub metadata_type: String,

    #[serde(skip)]
    pub m3u_id: Option<u64>,
}

#[derive(Debug, Clone)]
pub struct XtreamMetadata;

impl XtreamMetadata {
    pub async fn delete_by_m3u_id(&self, tx: &mut Connection, m3u_id: u64) -> Result<u64, Error> {
        let res = query_as!(
            u64,
            "delete xtream_metadata from xtream_metadata where m3u_id = ?",
            m3u_id
        )
        .execute(tx)
        .await?
        .rows_affected();

        Ok(res)
    }

    pub async fn get_latest_m3u_id(&self, tx: &mut Connection) -> Result<u64, Error> {
        let res: (u64,) = query_as("select max(m3u_id) from xtream_metadata")
            .fetch_one(tx)
            .await?;

        Ok(res.0)
    }

    pub async fn get_latest_by_type_and_m3u_id(
        &self,
        tx: &mut Connection,
        metadata_type: String,
        m3u_id: u64,
    ) -> Result<XtreamMetadataModel, Error> {
        let res = query_as!(
            XtreamMetadataModel,
            "select * from xtream_metadata where m3u_id = ? and metadata_type = ?",
            m3u_id,
            metadata_type
        )
        .fetch_one(tx)
        .await;

        res
    }
}

#[async_trait]
impl CRUD<XtreamMetadataModel, XtreamMetadataRequest> for XtreamMetadata {
    async fn get(&self, tx: &mut Connection, id: u64) -> Result<XtreamMetadataModel, Error> {
        let res = query_as!(
            XtreamMetadataModel,
            "select id, metadata, metadata_type, m3u_id from xtream_metadata where id = ?",
            id
        )
        .fetch_one(tx)
        .await;

        res
    }

    async fn insert(
        &self,
        tx: &mut Connection,
        xtream_metadata_request: XtreamMetadataRequest,
    ) -> Result<u64, Error> {
        let res = query_as!(
            XtreamMetadataModel,
            r#"insert into xtream_metadata (metadata, metadata_type, m3u_id) values (?, ?, ?)"#,
            xtream_metadata_request.metadata,
            xtream_metadata_request.metadata_type,
            xtream_metadata_request.m3u_id,
        )
        .execute(tx)
        .await?
        .last_insert_id();

        Ok(res)
    }

    async fn delete(&self, tx: &mut Connection, id: u64) -> Result<u64, Error> {
        let res = query_as!(u64, r#"delete from xtream_metadata where id = ?"#, id)
            .execute(tx)
            .await?
            .rows_affected();

        Ok(res)
    }
}
