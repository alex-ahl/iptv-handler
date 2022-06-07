use std::collections::HashMap;

use url::Url;

#[derive(Debug)]
pub struct M3U {
    pub extinfs: Vec<ExtInf>,
}

#[derive(Debug, Clone)]
pub struct ExtInf {
    pub name: String,
    pub attributes: HashMap<String, String>,
    pub url: Url,
    pub group_title: String,
}
