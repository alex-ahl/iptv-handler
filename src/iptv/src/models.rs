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
