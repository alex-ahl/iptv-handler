use futures_util::Stream;
use log::error;
use reqwest::{header::HeaderMap, Client, ClientBuilder, Error, Method, Response, Url};
use warp::hyper::body::Bytes;

#[derive(Clone)]
pub struct RestClient {
    client: Client,
}

impl RestClient {
    pub fn new() -> Self {
        let client: Client = ClientBuilder::new()
            .http1_only()
            .build()
            .expect("REST client created");

        Self { client }
    }

    pub async fn request(
        &self,
        method: Method,
        url: Url,
        headers: HeaderMap,
    ) -> Result<Response, Error> {
        let res = self
            .client
            .request(method, url)
            .headers(headers)
            .send()
            .await?;

        Ok(res)
    }
    pub async fn get(&self, url: &Url) -> Result<Response, Error> {
        let resp = self.client.get(url.to_string()).send().await;

        resp
    }

    pub async fn get_bytes(&self, url: &Url) -> Result<Bytes, Error> {
        let resp = self
            .client
            .get(url.to_string())
            .send()
            .await?
            .bytes()
            .await?;

        Ok(resp)
    }

    pub async fn get_bytes_stream(
        &self,
        url: &Url,
    ) -> Result<impl Stream<Item = Result<Bytes, Error>>, anyhow::Error> {
        let resp = self
            .client
            .get(url.to_string())
            .send()
            .await?
            .bytes_stream();

        Ok(resp)
    }

    pub async fn get_string(&self, url: &Url) -> String {
        let res = match self.client.get(url.to_string()).send().await {
            Ok(res) => res.text().await.unwrap_or_default(),
            Err(err) => {
                error!("Error getting string from request: {}", err.to_string());
                return "".to_string();
            }
        };

        res
    }
}
