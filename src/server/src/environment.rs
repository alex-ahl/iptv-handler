use api::models::xtream::XtreamConfig;
use envy::from_env;
use serde::Deserialize;
use url::Url;

pub fn init_env() -> Configuration {
    from_env::<Configuration>().expect("Correct environment variables not provided")
}

#[derive(Deserialize, Debug, Clone)]
pub struct Configuration {
    #[serde(default = "default_port")]
    pub port: u16,

    pub m3u: Url,
    pub database_url: String,

    #[serde(default = "default_backend_mode_only")]
    pub backend_mode_only: bool,

    #[serde(default = "env")]
    pub env: Environment,

    #[serde(default = "hourly_update_frequency")]
    pub hourly_update_frequency: u16,

    #[serde(default = "group_excludes")]
    pub group_excludes: Vec<String>,
    pub proxy_domain: String,

    #[serde(default = "use_xtream")]
    pub use_xtream: bool,

    #[serde(default = "xtream_config", flatten)]
    pub xtream_config: XtreamConfig,
}

fn default_port() -> u16 {
    3001
}

fn default_backend_mode_only() -> bool {
    true
}

fn use_xtream() -> bool {
    false
}

fn env() -> Environment {
    Environment::Development
}

fn hourly_update_frequency() -> u16 {
    12
}

fn group_excludes() -> Vec<String> {
    vec![]
}

pub fn xtream_config() -> XtreamConfig {
    XtreamConfig {
        xtream_base_domain: "".to_string(),
        xtream_username: "".to_string(),
        xtream_password: "".to_string(),

        xtream_proxied_username: "".to_string(),
        xtream_proxied_password: "".to_string(),
    }
}

#[derive(Deserialize, Debug, PartialEq, Clone)]
#[serde(rename_all = "lowercase")]
pub enum Environment {
    Development,
    Production,
}
