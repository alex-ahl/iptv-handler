use rest_client::RestClient;
use url::Url;

pub async fn get_m3u(url: &Url) -> Result<String, Box<dyn std::error::Error>> {
    let client = RestClient::new();
    let res = client.get_string(url).await;
    Ok(res)
}
