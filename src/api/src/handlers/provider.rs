use std::{convert::Infallible, sync::Arc};

use anyhow::Context;
use db::{models::ProviderRequest, services::provider::ProviderApiModel, DB};

use iptv::m3u::{
    parser::parse_m3u_url,
    tools::{count_channels, count_groups},
};
use log::{error, info};
use sqlx::{MySql, Transaction};
use url::Url;
use warp::{
    hyper::StatusCode,
    reply::{self, json, with_status, Response},
    Reply,
};

use crate::{
    models::{error::ApiError, provider::CreateProviderRequestApiModel},
    services::provider::{CreateProviderRequest, Service},
};

pub async fn list_providers(_db: Arc<DB>) -> Result<impl warp::Reply, Infallible> {
    Ok(warp::reply::json(&"".to_string()))
}

enum Conn<'a> {
    Tx(Transaction<'a, MySql>),
    Error(anyhow::Error),
}

pub async fn create_provider(
    provider: CreateProviderRequestApiModel,
    db: Arc<DB>,
) -> Result<Response, Infallible> {
    let tx = match db
        .pool
        .begin()
        .await
        .context("Could not initiate transaction")
    {
        Ok(tx) => Conn::Tx(tx),
        Err(e) => Conn::Error(e),
    };

    if let Conn::Error(e) = tx {
        error!("{}", e.root_cause());
        return Ok(
            reply::with_status("Connection error..", StatusCode::INTERNAL_SERVER_ERROR)
                .into_response(),
        );
    };

    if let Conn::Tx(mut tx) = tx {
        let url = match Url::parse(&provider.source)
            .context("Could not parse M3U URL, not a valid URL")
        {
            Ok(url) => url,
            Err(e) => {
                error!("{}", e.root_cause());
                return Ok(
                    reply::with_status(reply::json(&ApiError {}), StatusCode::BAD_REQUEST)
                        .into_response(),
                );
            }
        };

        let parsed_m3u = match parse_m3u_url(&url).await.context("Could not parse M3U") {
            Ok(m3u) => m3u,
            Err(e) => {
                error!("{}", e.root_cause());

                tx.rollback()
                    .await
                    .context("Could not rollback transaction")
                    .unwrap_or_default();

                return Ok(reply::with_status(
                    reply::json(&ApiError {}),
                    StatusCode::INTERNAL_SERVER_ERROR,
                )
                .into_response());
            }
        };

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
        };

        let res = db.create_provider(&mut tx, req).await;

        match res {
            Ok(res) => {
                tx.commit()
                    .await
                    .context("Could not commit transaction")
                    .unwrap_or_default();

                info!("Persisted {} extinf entries", extinf_entries_count);
                info!("Group count equals {}", group_count);

                return Ok(reply::json(&res).into_response());
            }
            Err(e) => {
                tx.rollback()
                    .await
                    .context("Could not rollback transaction")
                    .unwrap_or_default();

                error!("{}", e.root_cause());

                return Ok(reply::with_status(
                    reply::json(&ApiError {}),
                    StatusCode::INTERNAL_SERVER_ERROR,
                )
                .into_response());
            }
        };
    }

    Ok(
        reply::with_status(reply::json(&ApiError {}), StatusCode::INTERNAL_SERVER_ERROR)
            .into_response(),
    )
}

pub async fn get_provider(id: u64, db: Arc<DB>) -> Result<impl warp::Reply, Infallible> {
    let mut provider = ProviderApiModel::new();
    provider.initialize_db(db);

    let provider = provider.get_provider(id).await;

    if let Err(err) = provider {
        error!("{}", err);

        return Ok(
            with_status(json(&ApiError {}), StatusCode::INTERNAL_SERVER_ERROR).into_response(),
        );
    };

    Ok(reply::json(&provider.unwrap()).into_response())
}

pub async fn get_provider_entries_by_url(url: &str, db: Arc<DB>) -> Result<Response, Infallible> {
    let tx = match db
        .pool
        .begin()
        .await
        .context("Could not initiate transaction")
    {
        Ok(tx) => Conn::Tx(tx),
        Err(e) => Conn::Error(e),
    };

    if let Conn::Tx(mut tx) = tx {
        let provider = match db.provider.get_by_url(&mut tx, url).await {
            Ok(res) => res,
            Err(err) => {
                error!("Could not get provider with url\n{}\n{} ", url, err);
                Default::default()
            }
        };

        return Ok(json(&provider).into_response());
    };

    return Ok(with_status("Request failed", StatusCode::INTERNAL_SERVER_ERROR).into_response());
}

pub async fn provider_exists(url: &str, db: Arc<DB>) -> Result<Response, Infallible> {
    let tx = match db
        .pool
        .begin()
        .await
        .context("Could not initiate transaction")
    {
        Ok(tx) => Conn::Tx(tx),
        Err(e) => Conn::Error(e),
    };

    if let Conn::Tx(mut tx) = tx {
        let exists = db.provider.exists(&mut tx, url).await.unwrap_or_default();

        return Ok(json(&exists).into_response());
    };

    Ok(with_status("Request failed", StatusCode::INTERNAL_SERVER_ERROR).into_response())
}

pub async fn delete_provider(id: u64, db: Arc<DB>) -> Result<StatusCode, Infallible> {
    let mut provider = ProviderApiModel::new();
    provider.initialize_db(db);

    let res = match provider.delete(id).await {
        Ok(_) => {
            info!("Successfully deleted provider");
            StatusCode::OK
        }
        Err(_) => {
            error!("Failed to delete provider");
            StatusCode::BAD_REQUEST
        }
    };

    Ok(res)
}

pub async fn refresh_providers(db: Arc<DB>) -> Result<StatusCode, Infallible> {
    let status = match db.refresh_providers(db.clone()).await {
        Ok(_) => StatusCode::OK,
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR,
    };

    Ok(status)
}
