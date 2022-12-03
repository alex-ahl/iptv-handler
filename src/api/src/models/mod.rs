use serde::Deserialize;

pub mod error;

#[derive(Debug, Clone, Deserialize)]
pub struct CreateProviderRequestApiModel {
    pub name: Option<String>,
    pub source: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CreateM3uApiModel {
    pub provider_id: u64,
    pub group_excludes: Vec<String>,
    pub proxy_domain: String,
}
