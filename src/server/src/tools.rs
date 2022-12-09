use serde::de::DeserializeOwned;
use serde_json::from_slice;
use warp::hyper::{body, Body, Error, Response};

pub async fn deserialize_body<T>(body: Response<Body>) -> Result<T, Error>
where
    T: DeserializeOwned,
{
    let res = match body::to_bytes(body.into_body()).await {
        Ok(res) => Ok(from_slice::<T>(&res).unwrap()),
        Err(err) => Err(err),
    };

    res
}
