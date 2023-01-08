use db::{models::GroupRequest, services::provider::ExtInf};

#[derive(Debug, Clone, Default)]
pub struct ParsedM3u {
    pub extinfs: Vec<ExtInf>,
    pub groups: Vec<GroupRequest>,
}
