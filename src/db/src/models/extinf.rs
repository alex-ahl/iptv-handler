use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use sqlx::{Error, FromRow, query_as};

use crate::{Connection, CRUD};

#[derive(Debug, Clone)]
pub struct ExtInfRequest {
    pub name: String,
    pub url: String,
    pub track_id: Option<String>,
    pub prefix: Option<String>,
    pub extension: Option<String>,
    pub exclude: Option<bool>,

    pub m3u_id: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ExtInfModel {
    pub id: u64,
    pub name: String,
    pub url: String,
    pub track_id: Option<String>,
    pub prefix: Option<String>,
    pub extension: Option<String>,
    pub exclude: Option<bool>,

    #[serde(skip)]
    pub m3u_id: Option<u64>,
}

#[derive(Debug, Clone)]
pub struct ExtInf {}

impl ExtInf {
    pub async fn get_all_by_m3u(
        &self,
        tx: &mut Connection,
        m3u_id: u64,
    ) -> Result<Vec<ExtInfModel>, Error> {
        let res = query_as!(
            ExtInfModel,
            "select id, name, url, track_id, prefix, extension, exclude as `exclude: bool`, m3u_id from extinf where m3u_id = ?",
            m3u_id
        )
        .fetch_all(tx)
        .await;

        res
    }

    pub async fn delete_by_provider_id(
        &self,
        tx: &mut Connection,
        provider_id: u64,
    ) -> Result<u64, Error> {
        let res = query_as!(
            u64,
            "delete extinf from extinf 
            where m3u_id in (select id from m3u where provider_id = ?)",
            provider_id
        )
        .execute(tx)
        .await?
        .rows_affected();

        Ok(res)
    }

    pub async fn get_exclude_eligible_by_m3u_id(
        &self,
        tx: &mut Connection,
        m3u_id: u64,
        prefix: String
    ) -> Result<Vec<ExtInfModel>, Error> {
        let res = query_as!(
            ExtInfModel,
            r#"select id, name, url, track_id, prefix, extension, exclude as `exclude: bool`, m3u_id
            from extinf 
            where exclude = 1 and prefix = ? and m3u_id = ?"#,
            prefix,
            m3u_id
        )
        .fetch_all(tx)
        .await?;

        Ok(res)
    }

    pub async fn get_by_track_id(
        &self,
        tx: &mut Connection,
        m3u_id: u64,
        track_id: u64
    ) -> Result<ExtInfModel, Error> {
        let res = query_as!(
            ExtInfModel,
            "select id, name, url, track_id, prefix, extension, exclude as `exclude: bool`, m3u_id from extinf where m3u_id = ? and track_id = ?",
            m3u_id, 
            track_id
        )
        .fetch_one(tx)
        .await;

        res
    }
}

#[async_trait]
impl CRUD<ExtInfModel, ExtInfRequest> for ExtInf {
    async fn get(&self, tx: &mut Connection, id: u64) -> Result<ExtInfModel, Error> {
        let res = sqlx::query_as!(ExtInfModel, "select id, name, url, track_id, prefix, extension, exclude as `exclude: bool`, m3u_id from extinf where id = ?", id)
            .fetch_one(tx)
            .await;

        res
    }

    async fn insert(&self, tx: &mut Connection, extinf: ExtInfRequest) -> Result<u64, Error> {
        let res = sqlx::query_as!(
            ExtInfModel,
            r#"insert into extinf (name, url, prefix, track_id, extension, exclude, m3u_id) values (?, ?, ?, ?, ?, ?, ?)"#,
            extinf.name,
            extinf.url,
            extinf.prefix, 
            extinf.track_id, 
            extinf.extension,
            extinf.exclude,
            extinf.m3u_id,
        )
        .execute(tx)
        .await?
        .last_insert_id();

        Ok(res)
    }

    async fn delete(&self, tx: &mut Connection, id: u64) -> Result<u64, Error> {
        let res = sqlx::query_as!(u64, r#"delete from extinf where id = ?"#, id)
            .execute(tx)
            .await?
            .rows_affected();

        Ok(res)
    }
}
