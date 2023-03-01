use std::{convert::Infallible, sync::Arc};

use db::DB;
use log::error;
use rest_client::RestClient;
use warp::{
    http::HeaderMap,
    hyper::{Body, Response, StatusCode},
    path::FullPath,
    reply::with_status,
    Reply,
};

use crate::{
    models::{
        xtream::{Action, OptionalParams, TypeOutput},
        ApiConfiguration, Path,
    },
    services::xtream::XtreamService,
};

#[derive(Clone)]
pub struct XtreamHandler {
    xtream_service: XtreamService,
}

impl XtreamHandler {
    pub fn new(config: ApiConfiguration, db: Arc<DB>, client: Arc<RestClient>) -> Self {
        XtreamHandler {
            xtream_service: XtreamService::new(config, db, client),
        }
    }

    pub async fn stream(
        self,
        path: Path,
        headers: HeaderMap,
    ) -> Result<Response<Body>, Infallible> {
        let res = match self.xtream_service.proxy_stream(path, headers).await {
            Ok(res) => res,
            Err(err) => {
                error!("Failed to proxy xtream request: {}", err);
                with_status("INTERNAL SERVER ERROR", StatusCode::INTERNAL_SERVER_ERROR)
                    .into_response()
            }
        };

        Ok(res)
    }

    pub async fn xmltv(self, path: FullPath) -> Result<Response<Body>, Infallible> {
        let res = match self.xtream_service.proxy_xmltv(path.as_str()).await {
            Ok(res) => res,
            Err(err) => {
                error!("Failed to proxy xtream request: {}", err);
                with_status("INTERNAL SERVER ERROR", StatusCode::INTERNAL_SERVER_ERROR)
                    .into_response()
            }
        };

        Ok(res)
    }

    pub async fn get_type_output(
        self,
        type_output: TypeOutput,
    ) -> Result<Response<Body>, Infallible> {
        let res = match self.xtream_service.proxy_type_output(type_output).await {
            Ok(res) => res,
            Err(err) => {
                error!("Failed to proxy xtream request: {}", err);
                with_status("INTERNAL SERVER ERROR", StatusCode::INTERNAL_SERVER_ERROR)
                    .into_response()
            }
        };

        Ok(res)
    }

    pub async fn player_api_action(
        self,
        action: Action,
        optional_params: OptionalParams,
        path: FullPath,
    ) -> Result<Response<Body>, Infallible> {
        let res = match self
            .xtream_service
            .proxy_action(path.as_str(), action, optional_params)
            .await
        {
            Ok(res) => res,
            Err(err) => {
                error!("Failed to proxy xtream request: {}", err);
                with_status("INTERNAL SERVER ERROR", StatusCode::INTERNAL_SERVER_ERROR)
                    .into_response()
            }
        };

        Ok(res)
    }

    pub async fn player_api_login(self, path: FullPath) -> Result<Response<Body>, Infallible> {
        let res = match self.xtream_service.proxy_login(path.as_str()).await {
            Ok(res) => res,
            Err(err) => {
                error!("Failed to proxy xtream request: {}", err);
                with_status("INTERNAL SERVER ERROR", StatusCode::INTERNAL_SERVER_ERROR)
                    .into_response()
            }
        };

        Ok(res)
    }
}
