use std::{convert::Infallible, sync::Arc};

use db::DB;
use rest_client::RestClient;
use warp::{filters::BoxedFilter, get, path, query, Filter, Rejection, Reply};

use crate::{
    filters::{
        with_config, with_db, with_rest_client, with_xtream_config,
        xtream::{xtream_param_auth, xtream_path_auth},
    },
    handlers::{self, handle_rejection},
    models::{
        xtream::{Action, OptionalParams, TypeOutput, XtreamConfig},
        ApiConfiguration, Path,
    },
};

pub fn xtream_routes(
    config: ApiConfiguration,
    client: Arc<RestClient>,
    db: Arc<DB>,
) -> impl Filter<Extract = impl Reply, Error = Infallible> + Clone {
    let player_base_url = warp::path!("player_api.php")
        .and(get())
        .and(xtream_param_auth(config.xtream.clone()))
        .boxed();

    player_api_action(
        player_base_url.clone(),
        config.clone(),
        client.clone(),
        db.clone(),
    )
    .or(get_type_output(config.xtream.clone()))
    .or(xmltv(config.xtream.clone(), db.clone(), client.clone()))
    .or(player_api_login(
        player_base_url,
        config.xtream.clone(),
        db.clone(),
        client.clone(),
    ))
    .or(stream_three_segment(
        config.clone(),
        db.clone(),
        client.clone(),
    ))
    .or(stream_four_segment(config, db.clone(), client))
    .recover(handle_rejection)
}

fn stream_three_segment(
    config: ApiConfiguration,
    db: Arc<DB>,
    client: Arc<RestClient>,
) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    path::param::<String>()
        .and(path::param::<String>())
        .and(path::param::<String>())
        .and(path::end())
        .and(get())
        .and(xtream_path_auth(config.xtream.clone()))
        .map(|seg1, seg2: String, id: String| Path {
            segment1: Some(seg1),
            segment2: Some(seg2),
            segment3: None,
            id,
        })
        .and(with_config(config))
        .and(with_db(db))
        .and(with_rest_client(client))
        .and_then(handlers::xtream::stream)
}

fn stream_four_segment(
    config: ApiConfiguration,
    db: Arc<DB>,
    client: Arc<RestClient>,
) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    path::param::<String>()
        .and(path::param::<String>())
        .and(path::param::<String>())
        .and(path::param::<String>())
        .and(path::end())
        .and(get())
        .and(xtream_path_auth(config.xtream.clone()))
        .map(|seg1, seg2: String, seg3: String, id: String| Path {
            segment1: Some(seg1),
            segment2: Some(seg2),
            segment3: Some(seg3),
            id,
        })
        .and(with_config(config.clone()))
        .and(with_db(db))
        .and(with_rest_client(client))
        .and_then(handlers::xtream::stream)
}

fn xmltv(
    xtream_config: XtreamConfig,
    db: Arc<DB>,
    client: Arc<RestClient>,
) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    warp::path!("xmltv.php")
        .and(get())
        .and(xtream_param_auth(xtream_config.clone()))
        .and(path::full())
        .and(with_xtream_config(xtream_config))
        .and(with_db(db))
        .and(with_rest_client(client))
        .and_then(handlers::xtream::xmltv)
}

fn get_type_output(
    xtream_config: XtreamConfig,
) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    warp::path!("get.php")
        .and(get())
        .and(xtream_param_auth(xtream_config.clone()))
        .and(query::<TypeOutput>())
        .and_then(handlers::xtream::get_type_output)
}

fn player_api_action(
    base_url: BoxedFilter<()>,
    config: ApiConfiguration,
    client: Arc<RestClient>,
    db: Arc<DB>,
) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    base_url
        .and(query::<Action>())
        .and(query::<OptionalParams>())
        .and(path::full())
        .and(with_config(config))
        .and(with_rest_client(client))
        .and(with_db(db))
        .and_then(handlers::xtream::player_api_action)
}

fn player_api_login(
    base_url: BoxedFilter<()>,
    xtream_config: XtreamConfig,
    db: Arc<DB>,
    client: Arc<RestClient>,
) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    base_url
        .and(path::full())
        .and(with_xtream_config(xtream_config))
        .and(with_db(db))
        .and(with_rest_client(client))
        .and_then(handlers::xtream::player_api_login)
}
