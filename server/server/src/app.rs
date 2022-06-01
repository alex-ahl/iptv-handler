use std::sync::Arc;

use api::handlers::provider::create_provider;
use db::{models::ProviderRequest, DB};
use url::Url;

pub async fn init_app(m3u: Url, db: Arc<DB>) {
    let _res = create_provider(
        ProviderRequest {
            name: "Provider".to_string(),
            source: m3u.to_string(),
            groups: "".to_string(),
            channels: "".to_string(),
        },
        db,
    )
    .await
    .expect("Provider created");
}
