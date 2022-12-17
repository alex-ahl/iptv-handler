use warp::{hyper::StatusCode, reject::Reject};

pub mod error;
pub mod provider;
pub mod xtream;

#[derive(Debug)]
pub struct Invalid {
    pub status_code: StatusCode,
}
impl Reject for Invalid {}
