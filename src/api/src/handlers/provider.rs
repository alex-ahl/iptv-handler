use std::{convert::Infallible, sync::Arc};

use db::{services::provider::ProviderDBService, DB};

use log::{error, info};
use reqwest::StatusCode;
use warp::{
    reply::{json, with_status, Response},
    Reply,
};

use crate::{
    models::{error::ApiError, provider::CreateProviderRequestApiModel},
    services::provider::ProviderService,
};

pub async fn create_provider(
    provider: CreateProviderRequestApiModel,
    group_excludes: Vec<String>,
    db: Arc<DB>,
) -> Result<Response, Infallible> {
    let mut provider_service = ProviderService::new();
    provider_service.initialize(db);

    let res = match provider_service
        .create_provider(group_excludes, &provider.source)
        .await
    {
        Ok(res) => res,
        Err(err) => {
            error!("{}", err);
            with_status("INTERNAL_SERVER ERROR", StatusCode::INTERNAL_SERVER_ERROR).into_response()
        }
    };

    Ok(res)
}

pub async fn get_provider(id: u64, db: Arc<DB>) -> Result<impl Reply, Infallible> {
    let mut provider = ProviderDBService::new();
    provider.initialize_db(db);

    let provider = provider.get_provider(id).await;

    if let Err(err) = provider {
        error!("{}", err);

        return Ok(
            with_status(json(&ApiError {}), StatusCode::INTERNAL_SERVER_ERROR).into_response(),
        );
    };

    Ok(json(&provider.unwrap()).into_response())
}

pub async fn get_provider_entries_by_url(url: &str, db: Arc<DB>) -> Result<Response, Infallible> {
    let mut provider_service = ProviderService::new();
    provider_service.initialize(db);

    let res = match provider_service.get_provider_entries_by_url(url).await {
        Ok(provider_entries) => provider_entries,
        Err(err) => {
            error!("{}", err);
            with_status("INTERNAL_SERVER ERROR", StatusCode::INTERNAL_SERVER_ERROR).into_response()
        }
    };

    Ok(res)
}

pub async fn provider_exists(url: &str, db: Arc<DB>) -> Result<Response, Infallible> {
    let mut provider_service = ProviderService::new();
    provider_service.initialize(db);

    let res = match provider_service.provider_exists(url).await {
        Ok(res) => res,
        Err(err) => {
            error!("{}", err);
            with_status("INTERNAL_SERVER ERROR", StatusCode::INTERNAL_SERVER_ERROR).into_response()
        }
    };

    Ok(res)
}

pub async fn delete_provider(id: u64, db: Arc<DB>) -> Result<StatusCode, Infallible> {
    let mut provider = ProviderDBService::new();
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

pub async fn refresh_providers(
    group_excludes: Vec<String>,
    db: Arc<DB>,
) -> Result<StatusCode, Infallible> {
    let mut provider_service = ProviderService::new();
    provider_service.initialize(db);

    let status = match provider_service.refresh_providers(group_excludes).await {
        Ok(_) => StatusCode::OK,
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR,
    };

    Ok(status)
}
