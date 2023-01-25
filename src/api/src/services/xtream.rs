use anyhow::{bail, ensure, Context, Error};
use db::{
    services::{group::GroupDBService, provider::ProviderDBService},
    DB,
};
use log::info;
use rest_client::RestClient;
use serde::{de::DeserializeOwned, Serialize};
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
            Action, ActionTypes, Categories, LiveStream, Login, OptionalParams, Series, TypeOutput,
            VodStream, XtreamConfig, XtreamUrl,
        },
        ApiConfiguration, Path, ResponseData,
    },
    utils::compose_json_response,
};

use super::{proxy::ProxyService, HasId};

pub struct XtreamService {
    xtream_config: Option<XtreamConfig>,
    db: Option<Arc<DB>>,
    client: Option<Arc<RestClient>>,
}

impl XtreamService {
    pub fn new() -> Self {
        XtreamService {
            xtream_config: None,
            db: None,
            client: None,
        }
    }

    pub fn initialize(
        &mut self,
        xtream_config: XtreamConfig,
        db: Arc<DB>,
        client: Arc<RestClient>,
    ) {
        self.xtream_config = Some(xtream_config);
        self.db = Some(db);
        self.client = Some(client);
    }

    pub async fn proxy_stream(
        &self,
        path: Path,
        config: ApiConfiguration,
    ) -> Result<Response<Body>, Error> {
        if let (Some(_xtream_config), Some(db), Some(client)) = (
            self.xtream_config.as_ref(),
            self.db.as_ref(),
            self.client.as_ref(),
        ) {
            let mut proxy_service = ProxyService::new();
            proxy_service.initialize(db.clone(), client.clone());

            let res = proxy_service.proxy_stream(path, config).await?;

            Ok(res)
        } else {
            bail!("xtream service not fully initialized")
        }
    }

    pub async fn proxy_xmltv(&self, full_path: &str) -> Result<Response<Body>, Error> {
        if let (Some(xtream_config), Some(client)) =
            (self.xtream_config.as_ref(), self.client.as_ref())
        {
            let cred_query = self.compose_credentials_query_string(xtream_config);
            let url = self.compose_xmltv_url(xtream_config, full_path, cred_query)?;

            let response = self
                .proxy_request_bytes(&url.original, client.clone())
                .await?;

            let status_code = response.status();

            info!("[{}] {} => {}", status_code, url.proxied, url.original);

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
                .proxy_request_json::<Login>(&url.original, client.to_owned())
                .await?;

            let status_code = res.status_code.clone();

            res.data.user_info.username = xtream_config.xtream_proxied_username.clone();
            res.data.user_info.password = xtream_config.xtream_proxied_password.clone();
            res.data.server_info.url = xtream_config.xtream_proxied_domain.clone().unwrap();
            res.data.server_info.port = 3001.to_string();
            res.data.server_info.rtmp_port = 3001.to_string();
            res.data.server_info.https_port = 3001.to_string();

            let response = compose_json_response(res)?;

            info!("[{}] {} => {}", status_code, url.proxied, url.original);

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

            let urls = self.compose_action_url(&xtream_config, full_path, query)?;
            let original_url = urls.original.clone();

            let response = match ActionTypes::from_str(action.as_str()) {
                Ok(ActionTypes::GetLiveStreams) => {
                    self.proxy_streams::<LiveStream>(
                        urls.original,
                        &m3u_url,
                        "live",
                        db,
                        client.clone(),
                    )
                    .await?
                }
                Ok(ActionTypes::GetVodStreams) => {
                    self.proxy_streams::<VodStream>(
                        urls.original,
                        &m3u_url,
                        "movie",
                        db,
                        client.clone(),
                    )
                    .await?
                }
                Ok(ActionTypes::GetSeries) => {
                    self.proxy_series(urls.original, m3u_url, db, client.clone())
                        .await?
                }
                Ok(ActionTypes::GetLiveCategories) => {
                    self.proxy_categories(urls.original, m3u_url, db, client.clone())
                        .await?
                }
                Ok(ActionTypes::GetVodCategories) => {
                    self.proxy_categories(urls.original, m3u_url, db, client.clone())
                        .await?
                }
                Ok(ActionTypes::GetSeriesCategories) => {
                    self.proxy_categories(urls.original, m3u_url, db, client.clone())
                        .await?
                }

                _ => {
                    self.proxy_request_bytes(&urls.original, client.clone())
                        .await?
                }
            };

            let status_code = response.status();

            info!("[{}] {} => {}", status_code, urls.proxied, original_url);

            Ok(response)
        } else {
            bail!("proxy service not fully initialized")
        }
    }

    async fn proxy_categories(
        &self,
        proxy_url: Url,
        m3u_url: Url,
        db: Arc<DB>,
        client: Arc<RestClient>,
    ) -> Result<Response<Body>, Error> {
        let mut json = self
            .proxy_request_json::<Categories>(&proxy_url, client.clone())
            .await
            .context("getting categories json")?;

        let mut provider_db_service = ProviderDBService::new();
        provider_db_service.initialize_db(db.clone());

        match provider_db_service
            .get_latest_provider_entry(m3u_url.as_str())
            .await
        {
            Some(latest_provider_entry) => {
                let mut group_service = GroupDBService::new();
                group_service.initialize_db(db);

                let groups = group_service
                    .get_groups(latest_provider_entry.id)
                    .await
                    .context("getting groups")?;

                let included_groups: Vec<String> = groups
                    .into_iter()
                    .filter(|group| !group.exclude)
                    .map(|group| group.name)
                    .collect();

                json.data
                    .retain(|group| included_groups.contains(&group.category_name));

                let res = compose_json_response(json)
                    .context("composing get categories json response")?;

                Ok(res)
            }
            None => bail!("No provider entry found"),
        }
    }

    async fn proxy_streams<T>(
        &self,
        proxy_url: Url,
        m3u_url: &Url,
        prefix: &str,
        db: Arc<DB>,
        client: Arc<RestClient>,
    ) -> Result<Response<Body>, Error>
    where
        T: DeserializeOwned + Send + Serialize + Clone + HasId,
    {
        let mut json = self
            .proxy_request_json::<Vec<T>>(&proxy_url, client.clone())
            .await
            .context("getting streams json")?;

        let mut provider_db_service = ProviderDBService::new();
        provider_db_service.initialize_db(db.clone());

        match provider_db_service
            .get_latest_provider_entry(m3u_url.as_str())
            .await
        {
            Some(latest_provider_entry) => {
                let excluded_extinfs_ids = provider_db_service
                    .get_exclude_eligible_by_m3u_id(latest_provider_entry.id, prefix, db)
                    .await?;

                let mut json_filtered = vec![];

                for stream in &mut json.data {
                    if !excluded_extinfs_ids.contains(&stream.get_set_id().to_string()) {
                        json_filtered.push(stream.to_owned())
                    }
                }

                json.data = json_filtered;

                let res = compose_json_response(json).context("composing streams json response")?;

                Ok(res)
            }
            None => bail!("No provider entry found"),
        }
    }

    async fn proxy_series(
        &self,
        proxy_url: Url,
        m3u_url: Url,
        db: Arc<DB>,
        client: Arc<RestClient>,
    ) -> Result<Response<Body>, Error> {
        let mut json = self
            .proxy_request_json::<Vec<Series>>(&proxy_url, client.clone())
            .await
            .context("getting series json")?;

        let mut provider_db_service = ProviderDBService::new();

        provider_db_service.initialize_db(db.clone());

        match provider_db_service
            .get_latest_provider_entry(m3u_url.as_str())
            .await
        {
            Some(latest_provider_entry) => {
                let mut group_service = GroupDBService::new();
                group_service.initialize_db(db);

                let excluded_groups = group_service
                    .get_excluded_groups(latest_provider_entry.id)
                    .await?;

                let mut json_filtered = vec![];

                for series in json.data {
                    if excluded_groups.clone().into_iter().find(|cat| {
                        series.category_id == cat.xtream_cat_id.unwrap_or_default().to_string()
                    }) == None
                    {
                        json_filtered.push(series)
                    }
                }

                json.data = json_filtered;

                let res = compose_json_response(json).context("composing series json response")?;

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

        let data = res.json::<T>().await.context("deserialize json body")?;

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
    ) -> Result<XtreamUrl, Error> {
        let original = self.parse_url(
            xtream_config.xtream_base_domain.clone(),
            full_path,
            Some(query.clone()),
        )?;

        let proxied = self.parse_url(
            xtream_config
                .xtream_proxied_domain
                .clone()
                .unwrap_or_default(),
            full_path,
            Some(query),
        )?;

        let urls = XtreamUrl { original, proxied };

        Ok(urls)
    }

    fn compose_login_url(
        &self,
        xtream_config: &XtreamConfig,
        full_path: &str,
    ) -> Result<XtreamUrl, Error> {
        let query = Some(format!(
            "?username={}&password={}",
            xtream_config.xtream_username, xtream_config.xtream_password
        ));

        let original = self.parse_url(
            xtream_config.xtream_base_domain.clone(),
            full_path,
            query.clone(),
        )?;

        let proxied = self.parse_url(
            xtream_config
                .xtream_proxied_domain
                .clone()
                .unwrap_or_default(),
            full_path,
            query,
        )?;

        let urls = XtreamUrl { original, proxied };

        Ok(urls)
    }

    fn compose_xmltv_url(
        &self,
        xtream_config: &XtreamConfig,
        full_path: &str,
        query: String,
    ) -> Result<XtreamUrl, Error> {
        let original = self.parse_url(
            xtream_config.xtream_base_domain.clone(),
            full_path,
            Some(query.clone()),
        )?;

        let proxied = self.parse_url(
            xtream_config
                .xtream_proxied_domain
                .clone()
                .unwrap_or_default(),
            full_path,
            Some(query),
        )?;

        let urls = XtreamUrl { original, proxied };

        Ok(urls)
    }

    fn parse_url(
        &self,
        domain: String,
        full_path: &str,
        query: Option<String>,
    ) -> Result<Url, Error> {
        let url = match query {
            Some(query) => {
                Url::parse(format!("{}{}{}{}", "http://", domain, full_path, query).as_str())?
            }
            None => Url::parse(format!("{}{}{}", "http://", domain, full_path).as_str())?,
        };

        Ok(url)
    }
}
