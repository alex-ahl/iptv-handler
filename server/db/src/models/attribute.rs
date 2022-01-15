use serde::{Deserialize, Serialize};
use sqlx::{Error, FromRow};

use crate::{ConnectionPool, CRUD};

#[derive(Debug, Clone)]
pub struct AttributeRequest {
    key: String,
    value: String,
    provider_id: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct AttributeModel {
    id: u64,
    key: String,
    value: String,
    provider_id: Option<u64>,
}

#[derive(Debug, Clone)]
pub struct Attribute {
    db: ConnectionPool,
}

impl Attribute {
    pub fn new(db: ConnectionPool) -> Attribute {
        Attribute { db }
    }
}

#[async_trait::async_trait]
impl CRUD<AttributeModel, AttributeRequest> for Attribute {
    async fn get(&self, id: u64) -> Result<AttributeModel, Error> {
        let res = sqlx::query_as!(AttributeModel, "select * from attribute where id = ?", id)
            .fetch_one(&self.db)
            .await;

        res
    }

    async fn insert(&self, attribute: AttributeRequest) -> Result<u64, Error> {
        let mut tx = self.db.begin().await?;

        let res = sqlx::query_as!(
            AttributeModel,
            r#"insert into attribute (`key`, `value`, provider_id) values (?, ?, ?)"#,
            attribute.key,
            attribute.value,
            attribute.provider_id
        )
        .execute(&mut tx)
        .await?
        .last_insert_id();

        tx.commit().await?;

        Ok(res)
    }

    async fn delete(&self, id: u64) -> Result<u64, Error> {
        let res = sqlx::query_as!(u64, r#"delete from attribute where id = ?"#, id)
            .execute(&self.db)
            .await?
            .rows_affected();

        Ok(res)
    }
}
