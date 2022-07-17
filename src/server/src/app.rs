use std::sync::Arc;

use api::{handlers::provider::create_provider, models::CreateProviderRequestApiModel};
use db::services::provider::ProviderApiModel;
use db::DB;
use iptv::m3u::builder::create_m3u_file;
use log::error;
use url::Url;
use warp::hyper::{body, StatusCode};

pub async fn init_app(m3u: Url, db: Arc<DB>) {
    let response = create_provider(
        CreateProviderRequestApiModel {
            name: None::<String>,
            source: m3u.to_string(),
        },
        db.clone(),
    )
    .await
    .expect("Could not create provider");

    let success = StatusCode::from_u16(response.status().as_u16())
        .unwrap_or_default()
        .is_success();

    let id = match &body::to_bytes(response.into_body()).await {
        Ok(res) => serde_json::from_slice::<u64>(res).unwrap_or_default(),
        Err(_) => {
            error!("Could not deserialize response body of create provider request");
            0
        }
    };

    if success && id > 0 {
        let mut provider = ProviderApiModel::new();
        provider.initialize_db(db);

        if let Ok(provider) = provider.get_provider(id).await {
            if let Err(err) = create_m3u_file(provider).await {
                error!(".m3u file created failed with {}", err)
            }
        }
    } else {
        error!("Could not create provider at this time")
    }
}
