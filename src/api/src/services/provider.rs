use anyhow::Context;
use db::Connection;
use db::{
    models::{AttributeRequest, ExtInfRequest, M3uRequest, ProviderRequest},
    CRUD, DB,
};
use iptv::m3u::models::M3U;
use sqlx::{MySql, Transaction};
use std::fmt::Debug;
use std::sync::Arc;
use warp::hyper::StatusCode;

use crate::handlers::provider::{create_provider, delete_provider};
use crate::models::CreateProviderRequestApiModel;

pub struct CreateProviderRequest {
    pub provider_request: ProviderRequest,
    pub m3u: M3U,
}

#[async_trait::async_trait]
pub trait Service<TReturn, TInsert>: Send + Sync + Debug {
    async fn create_provider(
        &self,
        tx: &mut Connection,
        create_provider_request: CreateProviderRequest,
    ) -> Result<TReturn, anyhow::Error>;

    async fn refresh_providers(&self, db: Arc<DB>) -> Result<TReturn, anyhow::Error>;
}

#[async_trait::async_trait]
impl Service<u64, anyhow::Error> for DB {
    async fn create_provider(
        &self,
        tx: &mut Connection,
        create_provider_request: CreateProviderRequest,
    ) -> Result<u64, anyhow::Error> {
        let provider_id = self
            .provider
            .insert(tx, create_provider_request.provider_request)
            .await?;

        let m3u_id = self.m3u.insert(tx, M3uRequest { provider_id }).await?;

        for extinf in create_provider_request.m3u.extinfs {
            let extinf_id = self
                .extinf
                .insert(
                    tx,
                    ExtInfRequest {
                        name: extinf.name,
                        url: extinf.url.to_string(),
                        m3u_id,
                    },
                )
                .await?;

            for attr in extinf.attributes {
                self.attribute
                    .insert(
                        tx,
                        AttributeRequest {
                            key: attr.0.to_string(),
                            value: attr.1.to_string(),
                            extinf_id,
                        },
                    )
                    .await?;
            }
        }

        Ok(provider_id)
    }

    async fn refresh_providers(&self, db: Arc<DB>) -> Result<u64, anyhow::Error> {
        let tx = match self
            .pool
            .begin()
            .await
            .context("Could not initiate transaction")
        {
            Ok(tx) => Conn::Tx(tx),
            Err(e) => Conn::Error(e),
        };

        if let Conn::Tx(mut tx) = tx {
            let providers = self
                .provider
                .get_all(&mut tx)
                .await
                .context("Error gettings providers")?;

            for provider in providers {
                match delete_provider(provider.id, db.clone()).await {
                    Ok(status) => {
                        if status == StatusCode::OK {
                            create_provider(
                                CreateProviderRequestApiModel {
                                    name: provider.name,
                                    source: provider.source,
                                },
                                db.clone(),
                            )
                            .await
                            .expect("Could not create provider");
                        }
                    }
                    Err(_) => (),
                };
            }
        }

        Ok(StatusCode::OK.as_u16().into())
    }
}

enum Conn<'a> {
    Tx(Transaction<'a, MySql>),
    Error(anyhow::Error),
}
