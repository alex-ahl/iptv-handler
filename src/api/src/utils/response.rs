use anyhow::Error;

use warp::http::response::Builder;
use warp::reply::with_status;
use warp::{
    hyper::{Body, Response},
    reply::json,
    Reply,
};

use serde::Serialize;

use crate::models::ResponseData;

#[derive(Default, Debug, Clone, PartialEq)]
pub struct ResponseUtil;

impl ResponseUtil {
    pub fn new() -> Self {
        ResponseUtil
    }

    pub async fn compose_base_response(&self, res: &reqwest::Response) -> Result<Builder, Error> {
        let status = res.status();
        let headers = res.headers();
        let mut builder = Response::builder();

        builder = builder.status(status);

        for (key, val) in headers.iter() {
            builder = builder.header(key, val);
        }

        Ok(builder)
    }

    pub async fn compose_proxy_stream_response(
        &self,
        res: reqwest::Response,
        response_builder: Builder,
    ) -> Result<Response<Body>, Error> {
        let bytes_stream = res.bytes_stream();
        let body = Body::wrap_stream(bytes_stream);

        let response = response_builder.body(body).into_response();

        Ok(response)
    }

    pub fn compose_json_response<T>(&self, res: ResponseData<T>) -> Result<Response<Body>, Error>
    where
        T: Serialize,
    {
        let mut response = with_status(json(&res.data), res.status_code).into_response();

        for (key, val) in res.headers.iter() {
            response.headers_mut().insert(key, val.to_owned());
        }

        Ok(response)
    }

    pub async fn compose_byte_response(
        &self,
        res: reqwest::Response,
        response_builder: Builder,
    ) -> Result<Response<Body>, Error> {
        let bytes = res.bytes().await?;

        let response = response_builder.body(bytes).into_response();

        Ok(response)
    }
}
