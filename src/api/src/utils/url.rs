use crate::models::{Path, Track};
use anyhow::{Context, Error};
use db::{
    models::{HlsUrlRequest, M3uModel},
    CRUD, DB,
};
use iptv::m3u::parser::{parse_extension, parse_track_id};
use reqwest::Url;
use std::{fmt::Write, sync::Arc};

#[derive(Default, Debug, Clone, PartialEq, Copy)]
pub struct UrlUtil;

impl UrlUtil {
    pub fn new() -> Self {
        UrlUtil
    }

    pub fn parse_url(
        self,
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

    pub fn compose_host(
        &self,
        url: &mut String,
        domain: String,
        port: Option<u16>,
    ) -> Result<(), Error> {
        write!(url, "http://{}", domain)?;

        if let Some(port) = port {
            write!(url, ":{}", port)?;
        }

        Ok(())
    }

    pub fn compose_track(&self, url: &mut String, track: Track) -> Result<(), Error> {
        write!(url, "/{}", track.id)?;

        if let Some(extension) = &track.extension {
            write!(url, ".{}", extension)?;
        }

        Ok(())
    }

    pub fn parse_track(self, id: String) -> Result<Track, Error> {
        let parsed_id = parse_track_id(&id)
            .unwrap_or_default()
            .parse::<u64>()
            .context("parsing track id")?;

        let extension = parse_extension(id);

        Ok(Track {
            id: parsed_id,
            extension,
        })
    }

    pub fn compose_proxy_stream_url(
        &self,
        path: Path,
        m3u: M3uModel,
        segment_two_replacement: Option<String>,
        segment_three_replacement: Option<String>,
    ) -> Result<Url, Error> {
        let mut url = String::new();
        let track = self.parse_track(path.id.clone())?;

        self.compose_host(&mut url, m3u.domain, m3u.port)?;

        if None == path.segment3 {
            self.compose_two_segment_url(
                &mut url,
                path,
                segment_two_replacement,
                segment_three_replacement,
            )?;
        } else {
            self.compose_three_segment_url(
                &mut url,
                path,
                segment_two_replacement,
                segment_three_replacement,
            )?;
        }

        self.compose_track(&mut url, track)?;

        let url = url.as_str();

        let url = Url::parse(url).context("cannot parse url")?;

        Ok(url)
    }

    pub fn compose_two_segment_url(
        &self,
        url: &mut String,
        path: Path,
        segment1_replacement: Option<String>,
        segment2_replacement: Option<String>,
    ) -> Result<(), Error> {
        if path.segment1.is_some() {
            match segment1_replacement {
                Some(segment) => write!(url, "/{}", segment)?,
                None => write!(url, "/{}", path.segment1.unwrap())?,
            }
        }

        if path.segment2.is_some() {
            match segment2_replacement {
                Some(segment) => write!(url, "/{}", segment)?,
                None => write!(url, "/{}", path.segment2.unwrap())?,
            }
        }

        Ok(())
    }

    pub fn compose_three_segment_url(
        &self,
        url: &mut String,
        path: Path,
        segment2_replacement: Option<String>,
        segment3_replacement: Option<String>,
    ) -> Result<(), Error> {
        if let Some(segment1) = &path.segment1 {
            write!(url, "/{}", segment1)?;
        }

        if path.segment2.is_some() {
            match segment2_replacement {
                Some(segment) => write!(url, "/{}", segment)?,
                None => write!(url, "/{}", path.segment2.unwrap())?,
            }
        }

        if path.segment3.is_some() {
            match segment3_replacement {
                Some(segment) => write!(url, "/{}", segment)?,
                None => write!(url, "/{}", path.segment3.unwrap())?,
            }
        }

        Ok(())
    }

    pub fn compose_final_response_url(&self, response_url: &Url) -> Result<String, Error> {
        let mut url = String::new();

        write!(url, "{}://", response_url.scheme()).context("writing scheme")?;

        if let Some(host) = response_url.host_str() {
            write!(url, "{}", host).context("writing host")?;
        }

        if let Some(port) = response_url.port() {
            write!(url, ":{}", port).context("writing host scheme")?;
        }

        Ok(url)
    }

    pub fn is_hls_stream(&self, url: String) -> bool {
        url.ends_with(".m3u8")
    }

    pub async fn persist_final_response_url(&self, url: &Url, db: Arc<DB>) -> Result<(), Error> {
        let mut tx = db.pool.begin().await?;

        db.hls_url.truncate(&mut tx).await?;

        let url = self.compose_final_response_url(url)?;

        db.hls_url.insert(&mut tx, HlsUrlRequest { url }).await?;

        Ok(())
    }
}
