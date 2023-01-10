use std::sync::Arc;

use anyhow::{bail, Error};
use serde::{Deserialize, Serialize};

use crate::{models::GroupModel, DB};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupDBService {
    #[serde(skip)]
    db: Option<Arc<DB>>,
}

impl GroupDBService {
    pub fn new() -> Self {
        GroupDBService { db: None }
    }

    pub fn initialize_db(&mut self, db: Arc<DB>) {
        self.db = Some(db);
    }

    pub async fn get_groups(&self, m3u_id: u64) -> Result<Vec<GroupModel>, Error> {
        if let Some(ref db) = self.db {
            let mut tx = db.pool.begin().await?;

            let groups = db.group.get_all(&mut tx, m3u_id).await?;

            Ok(groups)
        } else {
            bail!("DB has not yet been initialized")
        }
    }
}
