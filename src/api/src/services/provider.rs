use anyhow::{bail, Context, Error};
use db::{
    models::ProviderRequest,
    services::provider::{CreateProviderRequest, ProviderDBService},
    DB,
};
use iptv::m3u::parser::parse_m3u_url;
use iptv::m3u::tools::{count_channels, count_groups};
use reqwest::Url;
use std::sync::Arc;
use warp::hyper::{Body, Response, StatusCode};
use warp::reply::{json, with_status};
use warp::Reply;

use crate::handlers::provider::create_provider;
use crate::models::provider::CreateProviderRequestApiModel;

pub struct ProviderService {
    db: Option<Arc<DB>>,
}

impl ProviderService {
    pub fn new() -> Self {
        ProviderService { db: None }
    }

    pub fn initialize(&mut self, db: Arc<DB>) {
        self.db = Some(db);
    }

    pub async fn create_provider(
        &self,
        group_excludes: Vec<String>,
        provider_source: &String,
    ) -> Result<Response<Body>, Error> {
        if let Some(ref db) = self.db {
            let url =
                Url::parse(provider_source).context("Could not parse M3U URL, not a valid URL")?;

            let parsed_m3u = parse_m3u_url(&url, &group_excludes)
                .await
                .context("Could not parse M3U")?;

            let extinf_entries_count = count_channels(&parsed_m3u);
            let group_count = count_groups(&parsed_m3u);

            let req = CreateProviderRequest {
                provider_request: ProviderRequest {
                    name: None,
                    source: url.to_string(),
                    channels: Some(extinf_entries_count),
                    groups: Some(count_groups(&parsed_m3u)),
                },
                m3u: parsed_m3u,
                channel_count: extinf_entries_count,
                group_count,
            };

            let mut provider_db_service = ProviderDBService::new();
            provider_db_service.initialize_db(db.clone());

            let provider_id = provider_db_service.create_provider(req).await?;

            Ok(json(&provider_id).into_response())
        } else {
            bail!("DB not properly initialized")
        }
    }

    pub async fn refresh_providers(
        &self,
        group_excludes: Vec<String>,
    ) -> Result<u64, anyhow::Error> {
        if let Some(ref db) = self.db {
            {
                let mut tx = db
                    .pool
                    .begin()
                    .await
                    .context("Could not initiate transaction")?;

                let providers = db
                    .provider
                    .get_all(&mut tx)
                    .await
                    .context("Error gettings providers")?;

                for provider in providers {
                    create_provider(
                        CreateProviderRequestApiModel {
                            name: provider.name,
                            source: provider.source,
                        },
                        group_excludes.to_owned(),
                        db.clone(),
                    )
                    .await
                    .expect("Could not create provider");
                }
            }

            Ok(StatusCode::OK.as_u16().into())
        } else {
            bail!("Unable to initialize db");
        }
    }

    pub async fn get_provider_entries_by_url(&self, url: &str) -> Result<Response<Body>, Error> {
        if let Some(ref db) = self.db {
            let mut provider_db_service = ProviderDBService::new();
            provider_db_service.initialize_db(db.clone());

            let res = provider_db_service.get_provider_entries_by_url(url).await?;

            Ok(with_status(json(&res), StatusCode::OK).into_response())
        } else {
            bail!("Unable to initialize db");
        }
    }

    pub async fn provider_exists(&self, url: &str) -> Result<Response<Body>, Error> {
        if let Some(ref db) = self.db {
            let mut tx = db
                .pool
                .begin()
                .await
                .context("Could not initiate transaction")?;

            let exists = db.provider.exists(&mut tx, url).await.unwrap_or_default();

            Ok(json(&exists).into_response())
        } else {
            bail!("Unable to initialize db");
        }
    }
}
