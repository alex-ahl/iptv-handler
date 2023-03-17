use api::models::{xtream::XtreamConfig, ApiConfiguration};
use envy::from_env;
use serde::Deserialize;
use url::Url;

pub fn init_env() -> Configuration {
    let mut env = from_env::<Configuration>().expect("Correct environment variables not provided");
    env.xtream_proxied_domain = Some(env.proxy_domain.clone());

    env
}

pub fn map_api_configuration(config: Configuration) -> ApiConfiguration {
    ApiConfiguration {
        m3u_url: config.m3u,
        group_excludes: config.group_excludes,
        xtream: XtreamConfig {
            xtream_enabled: config.xtream_enabled,
            xtream_base_domain: config.xtream_base_domain,
            xtream_username: config.xtream_username,
            xtream_password: config.xtream_password,
            xtream_proxied_domain: config.xtream_proxied_domain,
            xtream_proxied_username: config.xtream_proxied_username,
            xtream_proxied_password: config.xtream_proxied_password,
        },
    }
}

#[derive(Deserialize, Debug, Clone)]
pub struct Configuration {
    #[serde(default = "default_port")]
    pub port: u16,

    pub m3u: Url,
    pub database_url: String,

    #[serde(default = "default_init_app")]
    pub init_app: bool,

    #[serde(default = "env")]
    pub env: Environment,

    #[serde(default = "hourly_update_frequency")]
    pub hourly_update_frequency: u16,

    #[serde(default = "group_excludes")]
    pub group_excludes: Vec<String>,

    pub proxy_domain: String,

    #[serde(default = "xtream_enabled")]
    pub xtream_enabled: bool,

    #[serde(default = "xtream_base_domain")]
    xtream_base_domain: String,

    #[serde(default = "xtream_username")]
    xtream_username: String,

    #[serde(default = "xtream_password")]
    xtream_password: String,

    #[serde(default = "xtream_proxied_domain")]
    xtream_proxied_domain: Option<String>,

    #[serde(default = "xtream_proxied_username")]
    xtream_proxied_username: String,

    #[serde(default = "xtream_proxied_password")]
    xtream_proxied_password: String,
}

fn default_port() -> u16 {
    3001
}

fn default_init_app() -> bool {
    true
}

fn xtream_enabled() -> bool {
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

fn xtream_base_domain() -> String {
    String::from("")
}

fn xtream_username() -> String {
    String::from("")
}

fn xtream_password() -> String {
    String::from("")
}

fn xtream_proxied_domain() -> Option<String> {
    None
}
fn xtream_proxied_username() -> String {
    String::from("")
}

fn xtream_proxied_password() -> String {
    String::from("")
}

#[derive(Deserialize, Debug, PartialEq, Clone)]
#[serde(rename_all = "lowercase")]
pub enum Environment {
    Development,
    Production,
}
