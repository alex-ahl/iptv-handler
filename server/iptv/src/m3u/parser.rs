use url::Url;

use super::fetcher::get_m3u;

pub async fn parse_m3u(url: Url) {
    let _res = match get_m3u(&url).await {
        Ok(res) => res,
        Err(e) => e.to_string(),
    };
}
