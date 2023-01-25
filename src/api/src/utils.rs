use anyhow::{Context, Error};

use iptv::m3u::parser::{parse_extension, parse_track_id};
use serde::Serialize;
use warp::{
    hyper::{Body, Response},
    reply::{json, with_status},
    Reply,
};

use crate::models::{ResponseData, Track};

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

pub fn parse_track(id: String) -> Result<Track, Error> {
    let parsed_id = parse_track_id(&id)
        .unwrap_or_default()
        .parse::<u64>()
        .context("parsing track id")?;

    let extension = parse_extension(id);

    Ok(Track {
        id: parsed_id,
        extension,
    })
}
