use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct ApiError;
impl ApiError {}

#[derive(Serialize)]
pub struct ErrorMessage {
    pub(crate) code: u16,
    pub(crate) message: String,
}
