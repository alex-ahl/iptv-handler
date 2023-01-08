use serde::{Deserialize, Serialize};
use sqlx::{query, query_as, Error, FromRow};

use crate::{Connection, CRUD};

#[derive(Debug, Clone, Deserialize)]
pub struct GroupRequest {
    pub name: String,
    pub exclude: bool,

    pub m3u_id: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow, Default)]
pub struct GroupModel {
    pub id: u64,
    pub name: String,
    pub exclude: bool,

    #[serde(skip)]
    pub m3u_id: Option<u64>,
}

#[derive(Debug, Clone)]
pub struct Group {}

impl Group {
    pub async fn get_all(&self, tx: &mut Connection) -> Result<Vec<GroupModel>, Error> {
        let res = query_as!(
            GroupModel,
            "select id, name, exclude as `exclude: bool`, m3u_id from `group`",
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
            "delete `group` from `group`
            where m3u_id in (select id from m3u where provider_id = ?)",
            provider_id
        )
        .execute(tx)
        .await?
        .rows_affected();

        Ok(res)
    }

    pub async fn get_all_excluded(&self, tx: &mut Connection) -> Result<Vec<GroupModel>, Error> {
        let res = query_as!(
            GroupModel,
            "select id, name, exclude as `exclude: bool`, m3u_id from `group` where exclude = 1",
        )
        .fetch_all(tx)
        .await;

        res
    }

    pub async fn exists(&self, tx: &mut Connection, url: &str) -> Result<bool, Error> {
        let res = query_as!(
            GroupModel,
            "select id, name, exclude as `exclude: bool`, m3u_id from `group` where name = ?",
            url
        )
        .fetch_one(tx)
        .await
        .is_ok();

        Ok(res)
    }

    pub async fn truncate(&self, tx: &mut Connection) -> Result<u64, Error> {
        let res = query!("truncate table `group`")
            .execute(tx)
            .await?
            .rows_affected();

        Ok(res)
    }
}

#[async_trait::async_trait]
impl CRUD<GroupModel, GroupRequest> for Group {
    async fn get(&self, tx: &mut Connection, id: u64) -> Result<GroupModel, Error> {
        let res = query_as!(
            GroupModel,
            "select id, name, exclude as `exclude: bool`, m3u_id from `group` where id = ?",
            id
        )
        .fetch_one(tx)
        .await;
        res
    }

    async fn insert(&self, tx: &mut Connection, group: GroupRequest) -> Result<u64, Error> {
        let res = query_as!(
            GroupModel,
            r#"insert into `group` (name, exclude, m3u_id) values (?, ?, ?)"#,
            group.name,
            group.exclude,
            group.m3u_id
        )
        .execute(tx)
        .await?
        .last_insert_id();

        Ok(res)
    }

    async fn delete(&self, tx: &mut Connection, id: u64) -> Result<u64, Error> {
        let res = sqlx::query_as!(u64, r#"delete from `group` where id = ?"#, id)
            .execute(tx)
            .await?
            .rows_affected();

        Ok(res)
    }
}
