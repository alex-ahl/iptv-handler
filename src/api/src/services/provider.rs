use db::Connection;
use db::{
    models::{AttributeRequest, ExtInfRequest, M3uRequest, ProviderRequest},
    CRUD, DB,
};
use iptv::m3u::models::M3U;
use std::fmt::Debug;

pub struct CreateProviderRequest {
    pub provider_request: ProviderRequest,
    pub m3u: M3U,
}

#[async_trait::async_trait]
pub trait Service<TReturn, TInsert>: Send + Sync + Debug {
    async fn create_provider(
        &self,
        tx: &mut Connection,
        create_provider_request: CreateProviderRequest,
    ) -> Result<TReturn, anyhow::Error>;
}

#[async_trait::async_trait]
impl Service<u64, anyhow::Error> for DB {
    async fn create_provider(
        &self,
        tx: &mut Connection,
        create_provider_request: CreateProviderRequest,
    ) -> Result<u64, anyhow::Error> {
        let provider_id = self
            .provider
            .insert(tx, create_provider_request.provider_request)
            .await?;

        let m3u_id = self.m3u.insert(tx, M3uRequest { provider_id }).await?;

        for extinf in create_provider_request.m3u.extinfs {
            let extinf_id = self
                .extinf
                .insert(
                    tx,
                    ExtInfRequest {
                        name: extinf.name,
                        url: extinf.url.to_string(),
                        m3u_id,
                    },
                )
                .await?;

            for attr in extinf.attributes {
                self.attribute
                    .insert(
                        tx,
                        AttributeRequest {
                            key: attr.0.to_string(),
                            value: attr.1.to_string(),
                            extinf_id,
                        },
                    )
                    .await?;
            }
        }

        Result::Ok(provider_id)
    }
}
