use anyhow::{bail, ensure, Context, Error};
use rest_client::RestClient;
use std::sync::Arc;
use url::Url;
use warp::{
    hyper::{Body, Response},
    Reply,
};

use crate::{
    handlers::m3u::get_latest_m3u_file,
    models::xtream::{Action, OptionalParams, TypeOutput, XtreamConfig},
};

pub struct XtreamService {
    xtream_config: Option<XtreamConfig>,
    client: Option<Arc<RestClient>>,
}

impl XtreamService {
    pub fn new() -> Self {
        XtreamService {
            xtream_config: None,
            client: None,
        }
    }

    pub fn initialize(&mut self, xtream_config: XtreamConfig, client: Arc<RestClient>) {
        self.xtream_config = Some(xtream_config);
        self.client = Some(client);
    }

    pub async fn proxy_xmltv(&self, full_path: &str) -> Result<Response<Body>, Error> {
        if let (Some(xtream_config), Some(client)) =
            (self.xtream_config.as_ref(), self.client.as_ref())
        {
            let cred_query = self.compose_credentials_query_string(xtream_config);
            let url = self.compose_xmltv_url(xtream_config, full_path, cred_query)?;

            let response = self.proxy_request(&url, client.clone()).await?;

            Ok(response)
        } else {
            bail!("proxy service not fully initialized")
        }
    }

    pub async fn proxy_type_output(
        &self,
        TypeOutput { type_, output }: TypeOutput,
    ) -> Result<Response<Body>, Error> {
        ensure!(
            type_ == "m3u_plus" && output == "m3u8",
            "only m3u supported"
        );

        let response = get_latest_m3u_file()
            .await
            .context("error getting m3u file")?;

        Ok(response)
    }

    pub async fn proxy_login(&self, full_path: &str) -> Result<Response<Body>, Error> {
        if let (Some(xtream_config), Some(client)) =
            (self.xtream_config.as_ref(), self.client.as_ref())
        {
            let url = self.compose_login_url(xtream_config, full_path)?;

            let response = self.proxy_request(&url, client.clone()).await?;

            Ok(response)
        } else {
            bail!("proxy service not fully initialized")
        }
    }

    pub async fn proxy_action(
        &self,
        full_path: &str,
        action: Action,
        optional_params: OptionalParams,
    ) -> Result<Response<Body>, Error> {
        if let (Some(xtream_config), Some(client)) =
            (self.xtream_config.as_ref(), self.client.as_ref())
        {
            let query = self.compose_action_query_string(xtream_config, action, optional_params);
            let url = self.compose_action_url(&xtream_config, full_path, query)?;

            let response = self.proxy_request(&url, client.clone()).await?;

            Ok(response)
        } else {
            bail!("proxy service not fully initialized")
        }
    }

    fn compose_credentials_query_string(&self, xtream_config: &XtreamConfig) -> String {
        let query = format!(
            "?username={}&password={}",
            xtream_config.xtream_username, xtream_config.xtream_password,
        );

        query
    }

    fn compose_action_query_string(
        &self,
        xtream_config: &XtreamConfig,
        action: Action,
        optional_params: OptionalParams,
    ) -> String {
        let mut query = format!(
            "?username={}&password={}&action={}",
            xtream_config.xtream_username, xtream_config.xtream_password, action.action
        );

        if let Some(category_id) = optional_params.category_id {
            query = format!("{}&category_id={}", query, category_id);
        }

        if let Some(vod_id) = optional_params.vod_id {
            query = format!("{}&vod_id={}", query, vod_id);
        }

        if let Some(series_id) = optional_params.series_id {
            query = format!("{}&series_id={}", query, series_id);
        }

        if let Some(stream_id) = optional_params.stream_id {
            query = format!("{}&stream_id={}", query, stream_id);
        }

        query
    }

    async fn proxy_request(
        &self,
        url: &Url,
        client: Arc<RestClient>,
    ) -> Result<Response<Body>, Error> {
        let res = client.get(&url).await.context("error on proxy")?;
        let headers = res.headers();
        let status = res.status();

        let mut builder = Response::builder().status(status);

        for (key, val) in headers.iter() {
            builder = builder.header(key, val);
        }

        let bytes = res.bytes().await.context("error getting proxy bytes")?;

        let response = builder.body(bytes).into_response();

        Ok(response)
    }

    fn compose_action_url(
        &self,
        xtream_config: &XtreamConfig,
        full_path: &str,
        query: String,
    ) -> Result<Url, Error> {
        let url = Url::parse(
            format!(
                "{}{}{}{}",
                "http://", xtream_config.xtream_base_domain, full_path, query
            )
            .as_str(),
        )?;

        Ok(url)
    }

    fn compose_login_url(
        &self,
        xtream_config: &XtreamConfig,
        full_path: &str,
    ) -> Result<Url, Error> {
        let url = Url::parse(
            format!(
                "http://{}{}?username={}&password={}",
                xtream_config.xtream_base_domain,
                full_path,
                xtream_config.xtream_username,
                xtream_config.xtream_password
            )
            .as_str(),
        )?;

        Ok(url)
    }

    fn compose_xmltv_url(
        &self,
        xtream_config: &XtreamConfig,
        full_path: &str,
        query: String,
    ) -> Result<Url, Error> {
        let url = Url::parse(
            format!(
                "http://{}{}{}",
                xtream_config.xtream_base_domain, full_path, query
            )
            .as_str(),
        )?;

        Ok(url)
    }
}
