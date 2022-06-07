use std::sync::Arc;

use api::{handlers::provider::create_provider, models::CreateProviderRequestApiModel};
use db::DB;
use url::Url;

pub async fn init_app(m3u: Url, db: Arc<DB>) {
    let _res = create_provider(
        CreateProviderRequestApiModel {
            name: None::<String>,
            source: m3u.to_string(),
        },
        db,
    )
    .await
    .expect("Provider created");
}
