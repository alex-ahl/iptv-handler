use serde::{Deserialize, Serialize};
use serde_json::Value;
use strum::{Display, EnumString};

extern crate strum;
#[derive(Debug, Deserialize, Clone)]
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

#[derive(Debug, Clone, Deserialize, Default, PartialEq)]
pub struct XtreamConfig {
    pub xtream_base_domain: String,
    pub xtream_username: String,
    pub xtream_password: String,

    pub xtream_proxied_domain: Option<String>,
    pub xtream_proxied_username: String,
    pub xtream_proxied_password: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Login {
    #[serde(rename = "user_info")]
    pub user_info: UserInfo,
    #[serde(rename = "server_info")]
    pub server_info: ServerInfo,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserInfo {
    pub username: String,
    pub password: String,
    pub message: String,
    pub auth: i64,
    pub status: String,
    #[serde(rename = "exp_date")]
    pub exp_date: String,
    #[serde(rename = "is_trial")]
    pub is_trial: String,
    #[serde(rename = "active_cons")]
    pub active_cons: String,
    #[serde(rename = "created_at")]
    pub created_at: String,
    #[serde(rename = "max_connections")]
    pub max_connections: String,
    #[serde(rename = "allowed_output_formats")]
    pub allowed_output_formats: Vec<String>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ServerInfo {
    pub url: String,
    pub port: String,
    #[serde(rename = "https_port")]
    pub https_port: String,
    #[serde(rename = "server_protocol")]
    pub server_protocol: String,
    #[serde(rename = "rtmp_port")]
    pub rtmp_port: String,
    pub timezone: String,
    #[serde(rename = "timestamp_now")]
    pub timestamp_now: i64,
    #[serde(rename = "time_now")]
    pub time_now: String,
    pub process: bool,
}

pub type Streams = Vec<Stream>;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Stream {
    pub num: i64,
    pub name: String,
    #[serde(rename = "stream_type")]
    pub stream_type: String,
    #[serde(rename = "stream_id")]
    pub stream_id: i64,
    #[serde(rename = "stream_icon")]
    pub stream_icon: String,
    #[serde(rename = "epg_channel_id")]
    pub epg_channel_id: Option<String>,
    pub added: String,
    #[serde(rename = "is_adult")]
    pub is_adult: String,
    #[serde(rename = "category_id")]
    pub category_id: Option<String>,
    #[serde(rename = "category_ids")]
    pub category_ids: Vec<i64>,
    #[serde(rename = "custom_sid")]
    pub custom_sid: Option<String>,
    #[serde(rename = "tv_archive")]
    pub tv_archive: i64,
    #[serde(rename = "direct_source")]
    pub direct_source: String,
    #[serde(rename = "tv_archive_duration")]
    pub tv_archive_duration: Value,
}

#[derive(Debug, Eq, PartialEq, Display, EnumString)]
#[strum(serialize_all = "snake_case")]
pub enum ActionTypes {
    GetLiveStreams,
}
