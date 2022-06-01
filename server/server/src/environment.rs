use envy::from_env;
use serde::Deserialize;
use url::Url;

pub fn init_env() -> Configuration {
    from_env::<Configuration>().expect("Correct environment variables not provided")
}

#[derive(Deserialize, Debug)]
pub struct Configuration {
    #[serde(default = "default_port")]
    pub port: u16,
    #[serde(default = "default_backend_mode_only")]
    pub backend_mode_only: bool,
    pub m3u: Url,
}

fn default_port() -> u16 {
    3001
}

fn default_backend_mode_only() -> bool {
    true
}
