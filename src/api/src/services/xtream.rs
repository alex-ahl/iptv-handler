use anyhow::{bail, ensure, Context, Error};
use db::{
    services::{group::GroupDBService, provider::ProviderDBService},
    CRUD, DB,
};

use log::info;
use reqwest::Method;
use rest_client::RestClient;
use serde::{de::DeserializeOwned, Serialize};
use std::sync::Arc;
use url::Url;
use warp::{
    http::HeaderMap,
    hyper::{Body, Response},
};

use std::str::FromStr;

use crate::{
    handlers::m3u::get_latest_m3u_file,
    models::{
        xtream::{
            Action, ActionTypes, Categories, LiveStream, Login, OptionalParams, Series, TypeOutput,
            VodStream, XtreamUrl,
        },
        ApiConfiguration, Path,
    },
    utils::{proxy::ProxyUtil, response::ResponseUtil, url::UrlUtil},
};

use super::HasId;

#[derive(Clone)]
pub struct XtreamService {
    provider_db_service: ProviderDBService,
    proxy_util: ProxyUtil,
    response_util: ResponseUtil,
    url_util: UrlUtil,
    config: ApiConfiguration,
    db: Arc<DB>,
    client: Arc<RestClient>,
}

impl XtreamService {
    pub fn new(config: ApiConfiguration, db: Arc<DB>, client: Arc<RestClient>) -> Self {
        let mut provider_db_service = ProviderDBService::new();
        provider_db_service.initialize_db(db.clone());

        XtreamService {
            provider_db_service,
            proxy_util: ProxyUtil::new(ResponseUtil::new(), db.clone(), client.clone()),
            response_util: ResponseUtil::new(),
            url_util: UrlUtil::new(),
            config,
            db,
            client,
        }
    }

    pub async fn proxy_stream(
        &self,
        path: Path,
        headers: HeaderMap,
    ) -> Result<Response<Body>, Error> {
        let mut tx = self.db.pool.begin().await?;

        match self
            .provider_db_service
            .get_latest_provider_entry(self.config.m3u_url.as_str())
            .await
        {
            Some(latest_provider_entry) => {
                let m3u = self
                    .db
                    .m3u
                    .get(&mut tx, latest_provider_entry.id.clone())
                    .await
                    .context(format!(
                        "Unable to get m3u entry with id: {}",
                        latest_provider_entry.id
                    ))?;

                let url = self.url_util.compose_proxy_stream_url(
                    path.clone(),
                    m3u.clone(),
                    Some(self.config.xtream.xtream_username.clone()),
                    Some(self.config.xtream.xtream_password.clone()),
                )?;

                let res = self.client.request(Method::GET, url, headers).await?;

                let builder = self.response_util.compose_base_response(&res).await?;

                let res = self
                    .response_util
                    .compose_proxy_stream_response(res, builder)
                    .await
                    .context("error proxying stream")?;

                return Ok(res);
            }
            None => bail!("Unable to init provider service"),
        }
    }

    pub async fn proxy_xmltv(&self, full_path: &str) -> Result<Response<Body>, Error> {
        let cred_query = self.compose_credentials_query_string();
        let url = self.compose_xmltv_url(full_path, cred_query)?;

        let response = self.proxy_util.proxy_request_bytes(&url.original).await?;

        let status_code = response.status();

        info!("[{}] {} => {}", status_code, url.proxied, url.original);

        Ok(response)
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
        let url = self.compose_login_url(full_path)?;

        let mut res = self
            .proxy_util
            .proxy_request_json::<Login>(&url.original)
            .await?;

        let status_code = res.status_code.clone();

        res.data.user_info.username = self.config.xtream.xtream_proxied_username.clone();
        res.data.user_info.password = self.config.xtream.xtream_proxied_password.clone();
        res.data.server_info.url = self.config.xtream.xtream_proxied_domain.clone().unwrap();
        res.data.server_info.port = 3001.to_string();
        res.data.server_info.rtmp_port = 3001.to_string();
        res.data.server_info.https_port = 3001.to_string();

        let response = self.response_util.compose_json_response(res)?;

        info!("[{}] {} => {}", status_code, url.proxied, url.original);

        Ok(response)
    }

    pub async fn proxy_action(
        &self,
        full_path: &str,
        Action { action }: Action,
        optional_params: OptionalParams,
    ) -> Result<Response<Body>, Error> {
        let query = self.compose_action_query_string(action.clone(), optional_params);

        let urls = self.compose_action_url(full_path, query)?;
        let original_url = urls.original.clone();

        let response = match ActionTypes::from_str(action.as_str()) {
            Ok(ActionTypes::GetLiveStreams) => {
                self.proxy_streams::<LiveStream>(urls.original, "live")
                    .await?
            }
            Ok(ActionTypes::GetVodStreams) => {
                self.proxy_streams::<VodStream>(urls.original, "movie")
                    .await?
            }
            Ok(ActionTypes::GetSeries) => self.proxy_series(urls.original).await?,
            Ok(ActionTypes::GetLiveCategories) => self.proxy_categories(urls.original).await?,
            Ok(ActionTypes::GetVodCategories) => self.proxy_categories(urls.original).await?,

            Ok(ActionTypes::GetSeriesCategories) => self.proxy_categories(urls.original).await?,

            _ => self.proxy_util.proxy_request_bytes(&urls.original).await?,
        };

        let status_code = response.status();

        info!("[{}] {} => {}", status_code, urls.proxied, original_url);

        Ok(response)
    }

    async fn proxy_categories(&self, proxy_url: Url) -> Result<Response<Body>, Error> {
        let mut json = self
            .proxy_util
            .proxy_request_json::<Categories>(&proxy_url)
            .await
            .context("getting categories json")?;

        match self
            .provider_db_service
            .get_latest_provider_entry(self.config.m3u_url.as_str())
            .await
        {
            Some(latest_provider_entry) => {
                let mut group_service = GroupDBService::new();
                group_service.initialize_db(self.db.clone());

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

                let res = self
                    .response_util
                    .compose_json_response(json)
                    .context("composing get categories json response")?;

                Ok(res)
            }
            None => bail!("No provider entry found"),
        }
    }

    async fn proxy_streams<T>(&self, proxy_url: Url, prefix: &str) -> Result<Response<Body>, Error>
    where
        T: DeserializeOwned + Send + Serialize + Clone + HasId,
    {
        let mut json = self
            .proxy_util
            .proxy_request_json::<Vec<T>>(&proxy_url)
            .await
            .context("getting streams json")?;

        match self
            .provider_db_service
            .get_latest_provider_entry(self.config.m3u_url.as_str())
            .await
        {
            Some(latest_provider_entry) => {
                let excluded_extinfs_ids = self
                    .provider_db_service
                    .get_exclude_eligible_by_m3u_id(
                        latest_provider_entry.id,
                        prefix,
                        self.db.clone(),
                    )
                    .await?;

                let mut json_filtered = vec![];

                for stream in &mut json.data {
                    if !excluded_extinfs_ids.contains(&stream.get_set_id().to_string()) {
                        json_filtered.push(stream.to_owned())
                    }
                }

                json.data = json_filtered;

                let res = self
                    .response_util
                    .compose_json_response(json)
                    .context("composing streams json response")?;

                Ok(res)
            }
            None => bail!("No provider entry found"),
        }
    }

    async fn proxy_series(&self, proxy_url: Url) -> Result<Response<Body>, Error> {
        let mut json = self
            .proxy_util
            .proxy_request_json::<Vec<Series>>(&proxy_url)
            .await
            .context("getting series json")?;

        match self
            .provider_db_service
            .get_latest_provider_entry(self.config.m3u_url.as_str())
            .await
        {
            Some(latest_provider_entry) => {
                let mut group_service = GroupDBService::new();
                group_service.initialize_db(self.db.clone());

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

                let res = self
                    .response_util
                    .compose_json_response(json)
                    .context("composing series json response")?;

                Ok(res)
            }
            None => bail!("No provider entry found"),
        }
    }

    fn compose_credentials_query_string(&self) -> String {
        let query = format!(
            "?username={}&password={}",
            self.config.xtream.xtream_username, self.config.xtream.xtream_password,
        );

        query
    }

    fn compose_action_query_string(
        &self,
        action: String,
        optional_params: OptionalParams,
    ) -> String {
        let mut query = format!(
            "?username={}&password={}&action={}",
            self.config.xtream.xtream_username, self.config.xtream.xtream_password, action
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

    fn compose_action_url(&self, full_path: &str, query: String) -> Result<XtreamUrl, Error> {
        let original = self.url_util.parse_url(
            self.config.xtream.xtream_base_domain.clone(),
            full_path,
            Some(query.clone()),
        )?;

        let proxied = self.url_util.parse_url(
            self.config
                .xtream
                .xtream_proxied_domain
                .clone()
                .unwrap_or_default(),
            full_path,
            Some(query),
        )?;

        let urls = XtreamUrl { original, proxied };

        Ok(urls)
    }

    fn compose_login_url(&self, full_path: &str) -> Result<XtreamUrl, Error> {
        let query = Some(format!(
            "?username={}&password={}",
            self.config.xtream.xtream_username, self.config.xtream.xtream_password
        ));

        let original = self.url_util.parse_url(
            self.config.xtream.xtream_base_domain.clone(),
            full_path,
            query.clone(),
        )?;

        let proxied = self.url_util.parse_url(
            self.config
                .xtream
                .xtream_proxied_domain
                .clone()
                .unwrap_or_default(),
            full_path,
            query,
        )?;

        let urls = XtreamUrl { original, proxied };

        Ok(urls)
    }

    fn compose_xmltv_url(&self, full_path: &str, query: String) -> Result<XtreamUrl, Error> {
        let original = self.url_util.parse_url(
            self.config.xtream.xtream_base_domain.clone(),
            full_path,
            Some(query.clone()),
        )?;

        let proxied = self.url_util.parse_url(
            self.config
                .xtream
                .xtream_proxied_domain
                .clone()
                .unwrap_or_default(),
            full_path,
            Some(query),
        )?;

        let urls = XtreamUrl { original, proxied };

        Ok(urls)
    }
}
