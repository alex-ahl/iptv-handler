#[macro_use]
extern crate serde_json;

pub mod filters {
    use warp::{self, body, path, post, Filter, Rejection, Reply};

    use super::{handlers, models::M3u};

    pub async fn m3us() -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
        create()
    }

    pub fn create() -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
        path!("m3u")
            .and(post())
            .and(json_body())
            .and_then(handlers::create_m3u_link)
    }

    fn json_body() -> impl Filter<Extract = (M3u,), Error = Rejection> + Clone {
        body::content_length_limit(1024 * 16).and(body::json())
    }
}

pub mod handlers {
    use db::DB;
    use rest_client::RestClient;
    use std::{convert::Infallible, sync::Arc};
    use warp::{http::StatusCode, Reply};

    use super::models::M3u;

    pub async fn root_handler(
        db: Arc<DB>,
        client: Arc<RestClient>,
    ) -> Result<impl warp::Reply, warp::Rejection> {
        let _db = db;
        let _client = client;

        Ok(warp::reply::json(json!({ "temp": 1 }).as_object().unwrap()))
    }

    pub async fn create_m3u_link(create: M3u) -> Result<impl Reply, Infallible> {
        println!("{:?}", create);
        Ok(StatusCode::CREATED)
    }
}

mod models {
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Deserialize, Serialize, Clone)]
    pub struct M3u {
        url: String,
        name: String,
    }
}
