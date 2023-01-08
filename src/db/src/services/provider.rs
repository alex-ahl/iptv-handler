use std::{collections::HashMap, sync::Arc};

use anyhow::{bail, Context, Error};
use log::{error, info};
use serde::{Deserialize, Serialize};
use url::Url;

use crate::{
    models::{
        AttributeModel, AttributeRequest, ExtInfModel, ExtInfRequest, GroupRequest, M3uModel,
        M3uRequest, ProviderModel, ProviderRequest,
    },
    CRUD, DB,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderDBService {
    #[serde(skip)]
    db: Option<Arc<DB>>,

    pub provider: Option<ProviderModel>,
    pub m3u: Option<M3uModel>,
    pub extinfs: Option<Vec<ExtInfApiModel>>,
}

pub struct CreateProviderRequest {
    pub provider_request: ProviderRequest,
    pub m3u: M3U,
    pub channel_count: u32,
    pub groups: Vec<GroupRequest>,
}

#[derive(Debug)]
pub struct M3U {
    pub extinfs: Vec<ExtInf>,
}

#[derive(Debug, Clone)]
pub struct ExtInf {
    pub name: String,
    pub attributes: HashMap<String, String>,
    pub url: Url,
    pub track_id: Option<String>,
    pub prefix: Option<String>,
    pub extension: Option<String>,
    pub group_title: String,
    pub exclude: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtInfApiModel {
    pub id: u64,
    pub name: String,
    pub url: String,
    pub exclude: bool,
    pub m3u_id: Option<u64>,
    pub attributes: Option<Vec<AttributeModel>>,
}

impl ProviderDBService {
    pub fn new() -> Self {
        ProviderDBService {
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
                    exclude: extinf.exclude.unwrap_or_default(),
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
            let deleted_groups = db.group.delete_by_provider_id(&mut tx, id).await;
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
                    info!("Deleting {} group entries", aff_rows);
                    deleted_groups
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

    pub async fn create_provider(&self, req: CreateProviderRequest) -> Result<u64, Error> {
        if let Some(ref db) = self.db {
            let mut tx = db.pool.begin().await?;

            let provider_id = db.provider.insert(&mut tx, req.provider_request).await?;

            let m3u_id = db.m3u.insert(&mut tx, M3uRequest { provider_id }).await?;

            for extinf in req.m3u.extinfs {
                let extinf_id = db
                    .extinf
                    .insert(
                        &mut tx,
                        ExtInfRequest {
                            name: extinf.name,
                            url: extinf.url.to_string(),
                            track_id: extinf.track_id,
                            prefix: extinf.prefix,
                            exclude: Some(extinf.exclude),
                            extension: extinf.extension,
                            m3u_id,
                        },
                    )
                    .await?;

                for attr in extinf.attributes {
                    db.attribute
                        .insert(
                            &mut tx,
                            AttributeRequest {
                                key: attr.0.to_string(),
                                value: attr.1.to_string(),
                                extinf_id,
                            },
                        )
                        .await?;
                }
            }

            for mut group in req.groups.to_owned() {
                group.m3u_id = Some(m3u_id);
                db.group.insert(&mut tx, group).await?;
            }

            tx.commit().await?;

            let groups = req.groups.into_iter();

            let excluded_groups = groups.clone().filter(|group| group.exclude).count();
            let included_groups = groups.filter(|group| !group.exclude).count();

            info!("Persisted {} extinf entries", req.channel_count);
            info!("Included group count equals {}", included_groups);
            info!("Excluded group count equals {}", excluded_groups);

            Ok(provider_id)
        } else {
            bail!("DB not initialized properly")
        }
    }

    pub async fn get_provider_entries_by_url(
        &self,
        url: &str,
    ) -> Result<Vec<ProviderModel>, Error> {
        if let Some(ref db) = self.db {
            let mut tx = db
                .pool
                .begin()
                .await
                .context("Could not initiate transaction")?;

            let res = db
                .provider
                .get_by_url(&mut tx, url)
                .await
                .context("getting provider entries")?;

            tx.commit().await?;

            Ok(res)
        } else {
            bail!("Unable to initialize db");
        }
    }

    pub async fn get_latest_provider_entry(&self, url: &str) -> Option<ProviderModel> {
        let res = self
            .get_provider_entries_by_url(url)
            .await
            .unwrap_or_default();

        let latest_provider_entry = res.last()?;

        Some(latest_provider_entry.to_owned())
    }

    pub async fn get_exclude_eligible_by_m3u_id(
        &self,
        m3u_id: u64,
        prefix: &str,
        db: Arc<DB>,
    ) -> Result<Vec<String>, Error> {
        let mut tx = db.pool.begin().await?;

        let excluded_extinfs_ids = db
            .extinf
            .get_exclude_eligible_by_m3u_id(&mut tx, m3u_id, prefix.to_string())
            .await
            .context(format!("Unable to get ext entry with ID: {}", m3u_id))?
            .into_iter()
            .map(|extinf: ExtInfModel| extinf.track_id.unwrap_or_default())
            .collect::<Vec<String>>();

        tx.commit().await?;

        Ok(excluded_extinfs_ids)
    }
}
