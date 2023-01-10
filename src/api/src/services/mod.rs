use crate::models::xtream::{LiveStream, VodStream};

pub(crate) mod provider;
pub(crate) mod proxy;
pub(crate) mod xtream;

pub trait HasId {
    fn get_set_id(&mut self) -> i64;
}

impl HasId for LiveStream {
    fn get_set_id(&mut self) -> i64 {
        self.id = self.stream_id;
        self.id
    }
}

impl HasId for VodStream {
    fn get_set_id(&mut self) -> i64 {
        self.id = self.stream_id;
        self.id
    }
}
