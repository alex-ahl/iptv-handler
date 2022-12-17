use std::{convert::Infallible, error::Error};

use warp::{
    body::BodyDeserializeError,
    hyper::StatusCode,
    reject::MethodNotAllowed,
    reply::{json, with_status},
    Rejection, Reply,
};

use crate::models::{error::ErrorMessage, Invalid};

pub mod m3u;
pub mod provider;
pub mod proxy;
pub mod root;
pub mod xtream;

pub async fn handle_rejection(err: Rejection) -> Result<impl Reply, Infallible> {
    let code;
    let message;

    if err.is_not_found() {
        code = StatusCode::NOT_FOUND;
        message = "NOT_FOUND";
    } else if let Some(invalid) = err.find::<Invalid>() {
        if invalid.status_code == StatusCode::FORBIDDEN {
            code = StatusCode::FORBIDDEN;
            message = "FORBIDDEN";
        } else {
            code = StatusCode::BAD_REQUEST;
            message = "BAD_REQUEST";
        }
    } else if let Some(e) = err.find::<BodyDeserializeError>() {
        message = match e.source() {
            Some(cause) => {
                if cause.to_string().contains("denom") {
                    "FIELD_ERROR: denom"
                } else {
                    "BAD_REQUEST"
                }
            }
            None => "BAD_REQUEST",
        };
        code = StatusCode::BAD_REQUEST;
    } else if let Some(_) = err.find::<MethodNotAllowed>() {
        code = StatusCode::METHOD_NOT_ALLOWED;
        message = "METHOD_NOT_ALLOWED";
    } else {
        code = StatusCode::INTERNAL_SERVER_ERROR;
        message = "UNHANDLED_REJECTION";
    }
    let json = json(&ErrorMessage {
        code: code.as_u16(),
        message: message.into(),
    });

    Ok(with_status(json, code))
}
