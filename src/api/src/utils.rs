use anyhow::Error;

use serde::Serialize;
use warp::{
    hyper::{Body, Response},
    reply::{json, with_status},
    Reply,
};

use crate::models::ResponseData;

pub fn compose_json_response<T>(res: ResponseData<T>) -> Result<Response<Body>, Error>
where
    T: Serialize,
{
    let mut response = with_status(json(&res.data), res.status_code).into_response();

    for (key, val) in res.headers.iter() {
        response.headers_mut().insert(key, val.to_owned());
    }

    Ok(response)
}
