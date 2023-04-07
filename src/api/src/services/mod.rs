use std::convert::TryInto;

use serde_json::Value;

use crate::models::xtream::{LiveStream, Series, SeriesInfo, VodStream};

pub(crate) mod provider;
pub(crate) mod proxy;
pub(crate) mod xtream;

pub trait HasId {
    fn get_set_id(&mut self) -> &Value;
}

impl HasId for LiveStream {
    fn get_set_id(&mut self) -> &Value {
        self.id = self.stream_id.clone().try_into().unwrap_or_default();
        &self.id
    }
}

impl HasId for VodStream {
    fn get_set_id(&mut self) -> &Value {
        self.id = self.stream_id.clone().try_into().unwrap_or_default();
        &self.id
    }
}

impl HasId for Series {
    fn get_set_id(&mut self) -> &Value {
        self.id = self.category_id.clone().unwrap_or_default();
        &self.id
    }
}

impl HasId for SeriesInfo {
    fn get_set_id(&mut self) -> &Value {
        self.id = self.info.category_id.clone().unwrap_or_default();
        &self.id
    }
}
