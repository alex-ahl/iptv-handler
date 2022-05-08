use rest_client::RestClient;
use url::Url;

pub async fn init_app(client: RestClient) {
    client
        .post(&Url::parse("").expect("Could not parse M3U URL"), "")
        .await
        .expect("");
}
