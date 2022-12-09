use std::sync::Arc;

use anyhow::bail;
use api::{
    handlers::{
        m3u::m3u_file_exist,
        provider::{create_provider, get_provider_entries_by_url, provider_exists},
    },
    models::CreateProviderRequestApiModel,
};
use chrono::{Duration, NaiveDateTime, Utc};
use db::DB;
use db::{models::ProviderModel, services::provider::ProviderApiModel};
use iptv::m3u::builder::create_m3u_file;
use log::{debug, error, info};
use url::Url;
use warp::hyper::StatusCode;

use crate::{
    environment::{Configuration, Environment},
    tools::deserialize_body,
};

pub async fn init_app(config: Configuration, db: Arc<DB>) {
    if is_existing_provider(&config.m3u, db.clone()).await {
        try_provider_update(config, db.clone()).await;
    } else {
        info!("Creating new provider..");
        let provider_id = create_new_provider(&config.m3u, db.clone()).await;

        create_m3u(
            provider_id,
            config.group_excludes,
            config.proxy_domain,
            db.clone(),
        )
        .await;
    }
}

pub async fn try_provider_update(config: Configuration, db: Arc<DB>) {
    let provider = get_provider(&config.m3u, db.clone())
        .await
        .unwrap_or_default();

    let created_date = get_created_date(provider.created_at);

    if should_update_provider(created_date, config.hourly_update_frequency)
        || config.env == Environment::Development
    {
        info!("Provider is out of date, refreshing..");
        let provider_id = create_new_provider(&config.m3u, db.clone()).await;

        create_m3u(
            provider_id,
            config.group_excludes,
            config.proxy_domain,
            db.clone(),
        )
        .await;
    } else {
        info!("Provider is up to date. Skipping update...");

        match m3u_file_exist().await.unwrap_or_default().status() {
            StatusCode::OK => {
                info!("m3u file exists..");

                if config.env == Environment::Development {
                    debug!("Creating file anyways since developing..");

                    create_m3u(
                        provider.id,
                        config.group_excludes,
                        config.proxy_domain,
                        db.clone(),
                    )
                    .await;
                }
            }
            _ => {
                info!("Creating new m3u file..");
                create_m3u(
                    provider.id,
                    config.group_excludes,
                    config.proxy_domain,
                    db.clone(),
                )
                .await;
            }
        };
    }
}

async fn create_m3u(
    provider_id: u64,
    group_excludes: Vec<String>,
    proxy_domain: String,
    db: Arc<DB>,
) {
    if provider_id > 0 {
        let mut provider = ProviderApiModel::new();

        provider.initialize_db(db);

        if let Ok(provider) = provider.get_provider(provider_id).await {
            if let Err(err) = create_m3u_file(provider, group_excludes, proxy_domain).await {
                error!(".m3u file created failed with {}", err)
            }
        }
    } else {
        error!("Could not create provider at this time")
    }
}

async fn create_new_provider(m3u: &Url, db: Arc<DB>) -> u64 {
    let response = create_provider(
        CreateProviderRequestApiModel {
            name: None::<String>,
            source: m3u.to_string(),
        },
        db.clone(),
    )
    .await
    .expect("Could not create provider");

    let id = deserialize_body::<u64>(response).await.unwrap_or_default();

    id
}

async fn is_existing_provider(m3u: &Url, db: Arc<DB>) -> bool {
    let response = provider_exists(m3u.as_str(), db.clone()).await.expect("");

    let exists = is_success(response.status())
        && deserialize_body::<bool>(response).await.unwrap_or_default();

    exists
}

async fn get_provider(m3u: &Url, db: Arc<DB>) -> Result<ProviderModel, anyhow::Error> {
    let response = get_provider_entries_by_url(m3u.as_str(), db.clone())
        .await
        .expect("Could not get provider created date");

    let provider = deserialize_body::<Vec<ProviderModel>>(response)
        .await
        .unwrap_or_default();

    if let Some(provider) = provider.first() {
        return Ok(provider.to_owned());
    } else {
        bail!("No provider entry exists")
    }
}

fn get_created_date(created_at: Option<NaiveDateTime>) -> NaiveDateTime {
    created_at.unwrap_or(NaiveDateTime::from_timestamp(1, 0))
}

fn is_success(status_code: StatusCode) -> bool {
    StatusCode::from_u16(status_code.as_u16())
        .unwrap_or_default()
        .is_success()
}

fn should_update_provider(created_date: NaiveDateTime, hourly_update_frequency: u16) -> bool {
    (created_date + Duration::hours(hourly_update_frequency.into())) < Utc::now().naive_utc()
}
