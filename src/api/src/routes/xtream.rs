use std::{convert::Infallible, sync::Arc};

use db::DB;
use rest_client::RestClient;
use warp::{
    filters::BoxedFilter, get, header::headers_cloned, path, query, Filter, Rejection, Reply,
};

use crate::{
    filters::{
        with_xtream_handler,
        xtream::{xtream_param_auth, xtream_path_auth},
    },
    handlers::{handle_rejection, xtream::XtreamHandler},
    models::{
        xtream::{Action, OptionalParams, TypeOutput},
        ApiConfiguration, Path,
    },
};

pub fn xtream_routes(
    config: ApiConfiguration,
    client: Arc<RestClient>,
    db: Arc<DB>,
) -> impl Filter<Extract = impl Reply, Error = Infallible> + Clone {
    let handler = XtreamHandler::new(config.clone(), db.clone(), client.clone());

    let player_base_url = warp::path!("player_api.php")
        .and(get())
        .and(xtream_param_auth(config.xtream.clone()))
        .boxed();

    let get_path_auth = xtream_path_auth(config.xtream.clone()).and(get()).boxed();
    let get_param_auth = xtream_param_auth(config.xtream.clone()).and(get()).boxed();

    player_api_action(handler.clone(), player_base_url.clone())
        .or(get_type_output(get_param_auth.clone(), handler.clone()))
        .or(xmltv(get_param_auth, handler.clone()))
        .or(player_api_login(handler.clone(), player_base_url))
        .or(stream_three_segment(get_path_auth.clone(), handler.clone()))
        .or(stream_four_segment(get_path_auth, handler.clone()))
        .recover(handle_rejection)
}

fn stream_three_segment(
    base_filter: BoxedFilter<()>,
    handler: XtreamHandler,
) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    base_filter
        .and(path::param::<String>())
        .and(path::param::<String>())
        .and(path::param::<String>())
        .and(path::end())
        .map(|seg1, seg2: String, id: String| Path {
            segment1: Some(seg1),
            segment2: Some(seg2),
            segment3: None,
            id,
        })
        .and(headers_cloned())
        .and(with_xtream_handler(handler))
        .and_then(|path, headers, handler: XtreamHandler| handler.stream(path, headers))
}

fn stream_four_segment(
    base_filter: BoxedFilter<()>,
    handler: XtreamHandler,
) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    base_filter
        .and(path::param::<String>())
        .and(path::param::<String>())
        .and(path::param::<String>())
        .and(path::param::<String>())
        .and(path::end())
        .map(|seg1, seg2: String, seg3: String, id: String| Path {
            segment1: Some(seg1),
            segment2: Some(seg2),
            segment3: Some(seg3),
            id,
        })
        .and(headers_cloned())
        .and(with_xtream_handler(handler))
        .and_then(|path, headers, handler: XtreamHandler| handler.stream(path, headers))
}

fn xmltv(
    base_filter: BoxedFilter<()>,
    handler: XtreamHandler,
) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    warp::path!("xmltv.php")
        .and(base_filter)
        .and(path::full())
        .and(with_xtream_handler(handler))
        .and_then(|path, handler: XtreamHandler| handler.xmltv(path))
}

fn get_type_output(
    base_filter: BoxedFilter<()>,
    handler: XtreamHandler,
) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    warp::path!("get.php")
        .and(base_filter)
        .and(query::<TypeOutput>())
        .and(with_xtream_handler(handler))
        .and_then(|type_output: TypeOutput, handler: XtreamHandler| {
            handler.get_type_output(type_output)
        })
}

fn player_api_action(
    handler: XtreamHandler,
    base_url: BoxedFilter<()>,
) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    base_url
        .and(query::<Action>())
        .and(query::<OptionalParams>())
        .and(path::full())
        .and(with_xtream_handler(handler))
        .and_then(|action, optional_params, path, handler: XtreamHandler| {
            handler.player_api_action(action, optional_params, path)
        })
}

fn player_api_login(
    handler: XtreamHandler,
    base_url: BoxedFilter<()>,
) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    base_url
        .and(path::full())
        .and(with_xtream_handler(handler))
        .and_then(|path, handler: XtreamHandler| handler.player_api_login(path))
}
