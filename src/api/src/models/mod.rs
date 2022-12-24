use reqwest::{header::HeaderMap, Url};
use warp::{hyper::StatusCode, reject::Reject};

use self::xtream::XtreamConfig;

pub mod error;
pub mod provider;
pub mod xtream;

#[derive(Debug)]
pub struct Invalid {
    pub status_code: StatusCode,
}
impl Reject for Invalid {}

#[derive(Default, Debug, Clone, PartialEq)]
pub struct ResponseData<T> {
    pub data: T,
    pub headers: HeaderMap,
    pub status_code: StatusCode,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ApiConfiguration {
    pub m3u_url: Url,
    pub group_excludes: Vec<String>,
    pub xtream: XtreamConfig,
}
