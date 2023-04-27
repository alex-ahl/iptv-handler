use db::{models::GroupRequest, services::provider::ExtInf};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default)]
pub struct ParsedM3u {
    pub extinfs: Vec<ExtInf>,
    pub groups: Vec<GroupRequest>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct XtreamCategory {
    #[serde(rename = "category_id")]
    pub category_id: String,
    #[serde(rename = "category_name")]
    pub category_name: String,
    #[serde(rename = "parent_id")]
    pub parent_id: i64,
}

#[derive(Debug, Clone, Deserialize, Default, PartialEq)]
pub struct XtreamConfig {
    pub enabled: bool,

    pub base_domain: String,
    pub username: String,
    pub password: String,
}

#[derive(PartialEq, Clone, Copy)]
pub enum M3uType {
    Ts,
    M3u8,
    Custom,
}

impl Default for M3uType {
    fn default() -> Self {
        M3uType::Custom
    }
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct IptvConfiguration {
    pub proxy_domain: String,
    pub xtream_username: String,
    pub xtream_password: String,
}
