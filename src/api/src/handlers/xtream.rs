use std::{convert::Infallible, sync::Arc};

use log::error;
use rest_client::RestClient;
use warp::{
    hyper::{Body, Response, StatusCode},
    path::FullPath,
    reply::with_status,
    Reply,
};

use crate::{
    models::xtream::{Action, OptionalParams, TypeOutput, XtreamConfig},
    services::xtream::XtreamService,
};

pub async fn xmltv(
    path: FullPath,
    xtream_config: XtreamConfig,
    client: Arc<RestClient>,
) -> Result<Response<Body>, Infallible> {
    let mut xtream_service = XtreamService::new();
    xtream_service.initialize(xtream_config, client);
    let res = match xtream_service.proxy_xmltv(path.as_str()).await {
        Ok(res) => res,
        Err(err) => {
            error!("Failed to proxy xtream request: {}", err);
            with_status("INTERNAL SERVER ERROR", StatusCode::INTERNAL_SERVER_ERROR).into_response()
        }
    };

    Ok(res)
}

pub async fn get_type_output(type_output: TypeOutput) -> Result<Response<Body>, Infallible> {
    let xtream_service = XtreamService::new();
    let res = match xtream_service.proxy_type_output(type_output).await {
        Ok(res) => res,
        Err(err) => {
            error!("Failed to proxy xtream request: {}", err);
            with_status("INTERNAL SERVER ERROR", StatusCode::INTERNAL_SERVER_ERROR).into_response()
        }
    };

    Ok(res)
}

pub async fn player_api_action(
    action: Action,
    optional_params: OptionalParams,
    path: FullPath,
    xtream_config: XtreamConfig,
    client: Arc<RestClient>,
) -> Result<Response<Body>, Infallible> {
    let mut xtream_service = XtreamService::new();
    xtream_service.initialize(xtream_config, client);

    let res = match xtream_service
        .proxy_action(path.as_str(), action, optional_params)
        .await
    {
        Ok(res) => res,
        Err(err) => {
            error!("Failed to proxy xtream request: {}", err);
            with_status("INTERNAL SERVER ERROR", StatusCode::INTERNAL_SERVER_ERROR).into_response()
        }
    };

    Ok(res)
}

pub async fn player_api_login(
    path: FullPath,
    xtream_config: XtreamConfig,
    client: Arc<RestClient>,
) -> Result<Response<Body>, Infallible> {
    let mut xtream_service = XtreamService::new();
    xtream_service.initialize(xtream_config, client);

    let res = match xtream_service.proxy_login(path.as_str()).await {
        Ok(res) => res,
        Err(err) => {
            error!("Failed to proxy xtream request: {}", err);
            with_status("INTERNAL SERVER ERROR", StatusCode::INTERNAL_SERVER_ERROR).into_response()
        }
    };

    Ok(res)
}
