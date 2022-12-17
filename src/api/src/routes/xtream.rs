use std::{convert::Infallible, sync::Arc};

use rest_client::RestClient;
use warp::{
    filters::BoxedFilter, get, hyper::StatusCode, path, query, reject::custom, Filter, Rejection,
    Reply,
};

use crate::{
    filters::{with_rest_client, with_xtream_config},
    handlers::{self, handle_rejection},
    models::xtream::{Action, Credentials, OptionalParams, TypeOutput, XtreamConfig},
};

use super::Invalid;

pub fn xtream_routes(
    client: Arc<RestClient>,
    xtream_config: XtreamConfig,
) -> impl Filter<Extract = impl Reply, Error = Infallible> + Clone {
    let player_base_url = path!("player_api.php")
        .and(get())
        .and(xtream_auth(xtream_config.clone()))
        .untuple_one()
        .boxed();

    player_api_action(
        player_base_url.clone(),
        xtream_config.clone(),
        client.clone(),
    )
    .or(get_type_output(xtream_config.clone(), client.clone()))
    .or(xmltv(xtream_config.clone(), client.clone()))
    .or(player_api_login(player_base_url, xtream_config, client))
    .recover(handle_rejection)
}

fn xmltv(
    xtream_config: XtreamConfig,
    client: Arc<RestClient>,
) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    path!("xmltv.php")
        .and(get())
        .and(xtream_auth(xtream_config.clone()))
        .untuple_one()
        .and(path::full())
        .and(with_xtream_config(xtream_config))
        .and(with_rest_client(client))
        .and_then(handlers::xtream::xmltv)
}

fn get_type_output(
    xtream_config: XtreamConfig,
    client: Arc<RestClient>,
) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    path!("get.php")
        .and(get())
        .and(xtream_auth(xtream_config.clone()))
        .untuple_one()
        .and(query::<TypeOutput>())
        .and_then(handlers::xtream::get_type_output)
}

fn player_api_action(
    base_url: BoxedFilter<()>,
    xtream_config: XtreamConfig,
    client: Arc<RestClient>,
) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    base_url
        .and(query::<Action>())
        .and(query::<OptionalParams>())
        .and(path::full())
        .and(with_xtream_config(xtream_config))
        .and(with_rest_client(client))
        .and_then(handlers::xtream::player_api_action)
}

fn player_api_login(
    base_url: BoxedFilter<()>,
    xtream_config: XtreamConfig,
    client: Arc<RestClient>,
) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    base_url
        .and(path::full())
        .and(with_xtream_config(xtream_config))
        .and(with_rest_client(client))
        .and_then(handlers::xtream::player_api_login)
}

fn xtream_auth(
    xtream_config: XtreamConfig,
) -> impl Filter<Extract = ((),), Error = Rejection> + Clone {
    query().and(with_xtream_config(xtream_config)).and_then(
        |credentials: Credentials, xtream_config: XtreamConfig| async move {
            if credentials.username == xtream_config.xtream_proxied_username
                && credentials.password == xtream_config.xtream_proxied_password
            {
                Ok(())
            } else {
                Err(custom(Invalid {
                    status_code: StatusCode::FORBIDDEN,
                }))
            }
        },
    )
}
