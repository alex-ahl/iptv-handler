use anyhow::{bail, ensure, Context, Error};
use db::{
    services::{group::GroupDBService, provider::ProviderDBService},
    DB,
};
use rest_client::RestClient;
use serde::de::DeserializeOwned;
use std::sync::Arc;
use url::Url;
use warp::{
    hyper::{Body, Response},
    Reply,
};

use std::str::FromStr;

use crate::{
    handlers::m3u::get_latest_m3u_file,
    models::{
        xtream::{
            Action, ActionTypes, Categories, Login, OptionalParams, Streams, TypeOutput,
            XtreamConfig,
        },
        ResponseData,
    },
    utils::compose_json_response,
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

    pub async fn proxy_stream(&self) -> Result<Response<Body>, Error> {
        if let (Some(_xtream_config), Some(_client)) =
            (self.xtream_config.as_ref(), self.client.as_ref())
        {
            let response = warp::reply::reply().into_response();
            Ok(response)
        } else {
            bail!("proxy service not fully initialized")
        }
    }

    pub async fn proxy_xmltv(&self, full_path: &str) -> Result<Response<Body>, Error> {
        if let (Some(xtream_config), Some(client)) =
            (self.xtream_config.as_ref(), self.client.as_ref())
        {
            let cred_query = self.compose_credentials_query_string(xtream_config);
            let url = self.compose_xmltv_url(xtream_config, full_path, cred_query)?;

            let response = self.proxy_request_bytes(&url, client.clone()).await?;

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
            type_ == "m3u_plus" && (output == "m3u8" || output == "ts" || output == "rmtp"),
            "only m3u8, ts or rmtm supported"
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

            let mut res = self
                .proxy_request_json::<Login>(&url, client.to_owned())
                .await?;

            res.data.user_info.username = xtream_config.xtream_proxied_username.clone();
            res.data.user_info.password = xtream_config.xtream_proxied_password.clone();
            res.data.server_info.url = xtream_config.xtream_proxied_domain.clone().unwrap();
            res.data.server_info.port = 3001.to_string();
            res.data.server_info.rtmp_port = 3001.to_string();
            res.data.server_info.https_port = 3001.to_string();

            let response = compose_json_response(res)?;

            Ok(response)
        } else {
            bail!("proxy service not fully initialized")
        }
    }

    pub async fn proxy_action(
        &self,
        full_path: &str,
        Action { action }: Action,
        optional_params: OptionalParams,
        m3u_url: Url,
        db: Arc<DB>,
    ) -> Result<Response<Body>, Error> {
        if let (Some(xtream_config), Some(client)) =
            (self.xtream_config.as_ref(), self.client.as_ref())
        {
            let query =
                self.compose_action_query_string(xtream_config, action.clone(), optional_params);

            let proxy_url = self.compose_action_url(&xtream_config, full_path, query)?;

            let response = match ActionTypes::from_str(action.as_str()) {
                Ok(ActionTypes::GetLiveStreams) => {
                    self.proxy_get_live_streams(proxy_url, &m3u_url, db, client.clone())
                        .await?
                }
                Ok(ActionTypes::GetLiveCategories) => {
                    self.proxy_get_live_categories(proxy_url, db, client.clone())
                        .await?
                }
                _ => self.proxy_request_bytes(&proxy_url, client.clone()).await?,
            };

            Ok(response)
        } else {
            bail!("proxy service not fully initialized")
        }
    }

    async fn proxy_get_live_categories(
        &self,
        proxy_url: Url,
        db: Arc<DB>,
        client: Arc<RestClient>,
    ) -> Result<Response<Body>, Error> {
        let mut json = self
            .proxy_request_json::<Categories>(&proxy_url, client.clone())
            .await
            .context("getting get_live_categories json")?;

        let mut group_service = GroupDBService::new();
        group_service.initialize_db(db);

        let groups = group_service.get_groups().await.context("getting groups")?;

        let included_groups: Vec<String> = groups
            .into_iter()
            .filter(|group| !group.exclude)
            .map(|group| group.name)
            .collect();

        json.data
            .retain(|group| !included_groups.contains(&group.category_name));

        let res =
            compose_json_response(json).context("composing get_live_categories json response")?;

        Ok(res)
    }

    async fn proxy_get_live_streams(
        &self,
        proxy_url: Url,
        m3u_url: &Url,
        db: Arc<DB>,
        client: Arc<RestClient>,
    ) -> Result<Response<Body>, Error> {
        let mut json = self
            .proxy_request_json::<Streams>(&proxy_url, client.clone())
            .await
            .context("getting get_live_streams json")?;

        let mut provider_db_service = ProviderDBService::new();
        provider_db_service.initialize_db(db.clone());

        match provider_db_service
            .get_latest_provider_entry(m3u_url.as_str())
            .await
        {
            Some(latest_provider_entry) => {
                let excluded_extinfs_ids = provider_db_service
                    .get_exclude_eligible_by_m3u_id(latest_provider_entry.id, "live", db)
                    .await?;

                json.data
                    .retain(|stream| !excluded_extinfs_ids.contains(&stream.stream_id.to_string()));

                let res = compose_json_response(json)
                    .context("composing get_live_streams json response")?;

                Ok(res)
            }
            None => bail!("No provider entry found"),
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
        action: String,
        optional_params: OptionalParams,
    ) -> String {
        let mut query = format!(
            "?username={}&password={}&action={}",
            xtream_config.xtream_username, xtream_config.xtream_password, action
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

    async fn proxy_request_json<T>(
        &self,
        url: &Url,
        client: Arc<RestClient>,
    ) -> Result<ResponseData<T>, Error>
    where
        T: DeserializeOwned + Send,
    {
        let res = client.get(&url).await.context("error on proxy")?;
        let headers = res.headers().clone();
        let status_code = res.status().clone();

        let data = res.json::<T>().await.context("deserialize login body")?;

        Ok(ResponseData {
            data,
            headers,
            status_code,
        })
    }

    async fn proxy_request_bytes(
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
