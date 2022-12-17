use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct Credentials {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Deserialize)]
pub struct Action {
    pub action: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct TypeOutput {
    #[serde(rename = "type")]
    pub type_: String,
    pub output: String,
}

#[derive(Debug, Deserialize)]
pub struct OptionalParams {
    pub category_id: Option<String>,
    pub series_id: Option<String>,
    pub vod_id: Option<String>,
    pub stream_id: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct XtreamConfig {
    pub xtream_base_domain: String,
    pub xtream_username: String,
    pub xtream_password: String,

    pub xtream_proxied_username: String,
    pub xtream_proxied_password: String,
}
