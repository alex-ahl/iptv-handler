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
    #[serde(default = "default_backend_mode_only")]
    pub backend_mode_only: bool,
    pub m3u: Url,
    pub database_url: String,
    #[serde(default = "env")]
    pub env: Environment,
    #[serde(default = "hourly_update_frequency")]
    pub hourly_update_frequency: u16,
    #[serde(default = "group_excludes")]
    pub group_excludes: Vec<String>,
}

fn default_port() -> u16 {
    3001
}

fn default_backend_mode_only() -> bool {
    true
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

#[derive(Deserialize, Debug, PartialEq, Clone)]
#[serde(rename_all = "lowercase")]
pub enum Environment {
    Development,
    Production,
}
