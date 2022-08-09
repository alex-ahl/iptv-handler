use std::sync::Arc;

use anyhow::{bail, Context};
use log::{error, info};
use serde::{Deserialize, Serialize};

use crate::{
    models::{AttributeModel, M3uModel, ProviderModel},
    CRUD, DB,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderApiModel {
    #[serde(skip)]
    db: Option<Arc<DB>>,

    pub provider: Option<ProviderModel>,
    pub m3u: Option<M3uModel>,
    pub extinfs: Option<Vec<ExtInfApiModel>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtInfApiModel {
    pub id: u64,
    pub name: String,
    pub url: String,
    pub m3u_id: Option<u64>,
    pub attributes: Option<Vec<AttributeModel>>,
}

impl ProviderApiModel {
    pub fn new() -> Self {
        ProviderApiModel {
            db: None,
            provider: None,
            m3u: None,
            extinfs: None,
        }
    }

    pub fn initialize_db(&mut self, db: Arc<DB>) {
        self.db = Some(db);
    }

    pub async fn get_provider(mut self, id: u64) -> Result<Self, anyhow::Error> {
        if let Some(ref db) = self.db {
            let mut tx = db.pool.begin().await?;

            let provider = db
                .provider
                .get(&mut tx, id)
                .await
                .context("Could not get provider with id")?;

            let m3u = db
                .m3u
                .get(&mut tx, id)
                .await
                .context("Could not get provider with id")?;

            let extinfs = db
                .extinf
                .get_all_by_m3u(&mut tx, id)
                .await
                .context("Could not get provider with id")?;

            let mut extinf_models = vec![];

            for extinf in extinfs {
                let attr = db.attribute.get_all_by_extinf_id(&mut tx, extinf.id).await;

                extinf_models.push(ExtInfApiModel {
                    id: extinf.id,
                    name: extinf.name,
                    url: extinf.url,
                    m3u_id: extinf.m3u_id,
                    attributes: Some(attr.unwrap()),
                });
            }

            tx.commit().await.context("Could not close transaction")?;

            self.provider = Some(provider);
            self.m3u = Some(m3u);
            self.extinfs = Some(extinf_models);

            Ok(self)
        } else {
            bail!("DB has not yet been initialized")
        }
    }

    pub async fn delete(self, id: u64) -> Result<(), anyhow::Error> {
        if let Some(ref db) = self.db {
            let mut tx = db.pool.begin().await?;

            let deleted_attributes = db.attribute.delete_by_provider_id(&mut tx, id).await;
            let deleted_extinfs = db.extinf.delete_by_provider_id(&mut tx, id).await;
            let deleted_m3us = db.m3u.delete_by_provider_id(&mut tx, id).await;
            let deleted_provider = db.provider.delete(&mut tx, id).await;

            match deleted_attributes
                .and_then(|aff_rows| {
                    info!("Deleting {} attributes", aff_rows);
                    deleted_extinfs
                })
                .and_then(|aff_rows| {
                    info!("Deleting {} extinf entries", aff_rows);
                    deleted_m3us
                })
                .and_then(|aff_rows| {
                    info!("Deleting {} m3u entries", aff_rows);
                    deleted_provider
                }) {
                Err(err) => {
                    error!("Failed to delete provider\n{}\nRolling back..", err);

                    tx.rollback().await?;
                }
                Ok(aff_rows) => {
                    info!("Deleting {} provider", aff_rows);

                    tx.commit().await?
                }
            }
        }

        Ok(())
    }
}
