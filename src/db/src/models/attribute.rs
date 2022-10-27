use serde::{Deserialize, Serialize};
use sqlx::{Error, FromRow};

use crate::{Connection, CRUD};

#[derive(Debug, Clone)]
pub struct AttributeRequest {
    pub key: String,
    pub value: String,
    pub extinf_id: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct AttributeModel {
    pub id: u64,
    pub key: String,
    pub value: String,

    #[serde(skip)]
    pub extinf_id: Option<u64>,
}

#[derive(Debug, Clone)]
pub struct Attribute {}

impl Attribute {
    pub async fn get_all_by_extinf_id(
        &self,
        tx: &mut Connection,
        extinf_id: u64,
    ) -> Result<Vec<AttributeModel>, Error> {
        let res = sqlx::query_as!(
            AttributeModel,
            "select * from attribute where extinf_id = ?",
            extinf_id
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
        let res = sqlx::query_as!(
            u64,
            "delete attribute from attribute
            join extinf on attribute.extinf_id = extinf.id
            where m3u_id in (select id from `m3u` where provider_id = ?)",
            provider_id
        )
        .execute(tx)
        .await?
        .rows_affected();

        Ok(res)
    }
}

#[async_trait::async_trait]
impl CRUD<AttributeModel, AttributeRequest> for Attribute {
    async fn get(&self, tx: &mut Connection, id: u64) -> Result<AttributeModel, Error> {
        let res = sqlx::query_as!(AttributeModel, "select * from attribute where id = ?", id)
            .fetch_one(tx)
            .await;

        res
    }

    async fn insert(&self, tx: &mut Connection, attribute: AttributeRequest) -> Result<u64, Error> {
        let res = sqlx::query_as!(
            AttributeModel,
            r#"insert into attribute (`key`, `value`, extinf_id) values (?, ?, ?)"#,
            attribute.key,
            attribute.value,
            attribute.extinf_id
        )
        .execute(tx)
        .await?
        .last_insert_id();

        Ok(res)
    }

    async fn delete(&self, tx: &mut Connection, id: u64) -> Result<u64, Error> {
        let res = sqlx::query_as!(u64, r#"delete from attribute where id = ?"#, id)
            .execute(tx)
            .await?
            .rows_affected();

        Ok(res)
    }
}
