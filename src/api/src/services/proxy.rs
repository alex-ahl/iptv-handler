use std::sync::Arc;

use anyhow::{bail, Context, Error};
use rest_client::RestClient;
use url::Url;
use warp::{
    http::response::Builder,
    hyper::{Body, Response},
    Reply,
};

use db::{models::M3uModel, services::provider::ProviderDBService, CRUD, DB};
use std::fmt::Write;

use crate::{
    models::{ApiConfiguration, Path, Track},
    utils::parse_track,
};

#[derive(Clone)]
pub struct ProxyService {
    db: Option<Arc<DB>>,
    client: Option<Arc<RestClient>>,
}

impl ProxyService {
    pub fn new() -> Self {
        ProxyService {
            db: None,
            client: None,
        }
    }

    pub fn initialize(&mut self, db: Arc<DB>, client: Arc<RestClient>) {
        self.db = Some(db);
        self.client = Some(client);
    }

    pub async fn proxy_stream(
        &self,
        path: Path,
        config: ApiConfiguration,
    ) -> Result<Response<Body>, Error> {
        if let Some(db) = self.db.as_ref() {
            let mut tx = db.pool.begin().await?;

            let mut provider_db_service = ProviderDBService::new();
            provider_db_service.initialize_db(db.clone());

            match provider_db_service
                .get_latest_provider_entry(config.m3u_url.as_str())
                .await
            {
                Some(latest_provider_entry) => {
                    let m3u = db
                        .m3u
                        .get(&mut tx, latest_provider_entry.id.clone())
                        .await
                        .context(format!(
                            "Unable to get m3u entry with id: {}",
                            latest_provider_entry.id
                        ))?;

                    let url = self.compose_proxy_stream_url(path, m3u, config)?;
                    println!("{:?}", url);
                    if let Some(ref client) = self.client {
                        let res = client.get(&url).await.context("error on proxy")?;
                        let builder = self.compose_base_response(&res).await?;

                        let res = self
                            .compose_proxy_stream_response(res, builder)
                            .await
                            .context("error proxying stream")?;

                        return Ok(res);
                    } else {
                        bail!("Unable to init rest clieent");
                    }
                }
                None => bail!("Unable to init provider service"),
            }
        } else {
            bail!("Unable to initialize db");
        }
    }

    fn compose_proxy_stream_url(
        &self,
        path: Path,
        m3u: M3uModel,
        config: ApiConfiguration,
    ) -> Result<Url, Error> {
        let mut url = String::new();

        let track = parse_track(path.id.clone())?;

        write!(&mut url, "http://{}", m3u.domain)?;

        if let Some(port) = m3u.port {
            write!(&mut url, ":{}", port)?;
        }

        if None == path.segment3 {
            self.compose_two_segment_url(&mut url, &track, &path, &config)?;
        } else {
            self.compose_three_segment_url(&mut url, &track, &path, &config)?;
        }

        let url = url.as_str();

        let url = Url::parse(url).context("cannot parse url")?;

        Ok(url)
    }

    fn compose_two_segment_url(
        &self,
        url: &mut String,
        track: &Track,
        path: &Path,
        config: &ApiConfiguration,
    ) -> Result<(), Error> {
        if let Some(segment1) = &path.segment1 {
            match config.xtream_enabled {
                true => write!(url, "/{}", config.xtream.xtream_username)?,
                false => write!(url, "/{}", segment1)?,
            };
        }

        if let Some(segment2) = &path.segment2 {
            match config.xtream_enabled {
                true => write!(url, "/{}", config.xtream.xtream_password)?,
                false => write!(url, "/{}", segment2)?,
            };
        }

        write!(url, "/{}", track.id)?;

        if let Some(extension) = &track.extension {
            write!(url, ".{}", extension)?;
        }

        Ok(())
    }

    fn compose_three_segment_url(
        &self,
        url: &mut String,
        track: &Track,
        path: &Path,
        config: &ApiConfiguration,
    ) -> Result<(), Error> {
        if let Some(segment1) = &path.segment1 {
            write!(url, "/{}", segment1)?;
        }

        if let Some(segment2) = &path.segment2 {
            match config.xtream_enabled {
                true => write!(url, "/{}", config.xtream.xtream_username)?,
                false => write!(url, "/{}", segment2)?,
            };
        }

        if let Some(segment3) = &path.segment3 {
            match config.xtream_enabled {
                true => write!(url, "/{}", config.xtream.xtream_password)?,
                false => write!(url, "/{}", segment3)?,
            };
        }

        write!(url, "/{}", track.id)?;

        if let Some(extension) = &track.extension {
            write!(url, ".{}", extension)?;
        }

        Ok(())
    }

    pub async fn proxy_attribute(&self, id: u64) -> Result<Response<Body>, Error> {
        if let Some(ref db) = self.db {
            let mut tx = db.pool.begin().await?;

            let attr = db
                .attribute
                .get(&mut tx, id)
                .await
                .context(format!("Unable to get attribute entry with ID: {}", id))?;

            let url = Url::parse(&attr.value)?;

            if let Some(ref client) = self.client {
                let res = client.get(&url).await.context("error on proxy")?;

                let builder = self.compose_base_response(&res).await?;

                let res = self
                    .compose_proxy_attribute_response(res, builder)
                    .await
                    .context("error proxying attribute")?;

                return Ok(res);
            } else {
                bail!("Unable to initialize REST client");
            }
        } else {
            bail!("Unable to initialize db");
        }
    }

    async fn compose_base_response(&self, res: &reqwest::Response) -> Result<Builder, Error> {
        let status = res.status();
        let headers = res.headers();
        println!("{:?}", status);
        let mut builder = Response::builder().status(status);

        for (key, val) in headers.iter() {
            builder = builder.header(key, val);
        }

        Ok(builder)
    }

    async fn compose_proxy_stream_response(
        &self,
        res: reqwest::Response,
        response_builder: Builder,
    ) -> Result<Response<Body>, Error> {
        let bytes_stream = res.bytes_stream();
        let body = Body::wrap_stream(bytes_stream);

        let response = response_builder.body(body).into_response();

        Ok(response)
    }

    async fn compose_proxy_attribute_response(
        &self,
        res: reqwest::Response,
        response_builder: Builder,
    ) -> Result<Response<Body>, Error> {
        let bytes = res.bytes().await?;

        let response = response_builder.body(bytes).into_response();

        Ok(response)
    }
}
