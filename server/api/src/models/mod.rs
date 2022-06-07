use serde::Deserialize;

pub mod error;

#[derive(Debug, Clone, Deserialize)]
pub struct CreateProviderRequestApiModel {
    pub name: Option<String>,
    pub source: String,
}
