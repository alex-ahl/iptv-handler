use warp::{hyper::StatusCode, path::FullPath, query, reject::custom, Filter, Rejection};

use crate::{
    filters::with_xtream_config,
    models::{
        xtream::{Credentials, XtreamConfig},
        Invalid,
    },
};

pub fn xtream_param_auth(
    xtream_config: XtreamConfig,
) -> impl Filter<Extract = (), Error = Rejection> + Clone {
    query()
        .and(with_xtream_config(xtream_config))
        .and_then(
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
        .untuple_one()
}

pub fn xtream_path_auth(
    xtream_config: XtreamConfig,
) -> impl Filter<Extract = (), Error = Rejection> + Clone {
    warp::any()
        .and(warp::path::full())
        .and(with_xtream_config(xtream_config))
        .map(|path: FullPath, xtream_config: XtreamConfig| {
            let path = path.as_str();
            let path_segments = match path.starts_with("/series")
                || path.starts_with("/movies")
                || path.starts_with("/live")
            {
                true => path.split('/').skip(2).take(2).map(String::from).collect(),
                false => path.split('/').skip(1).take(2).map(String::from).collect(),
            };

            (path_segments, xtream_config)
        })
        .and_then(
            |(credentials, xtream_config): (Vec<String>, XtreamConfig)| async move {
                if credentials.first().unwrap().to_owned() == xtream_config.xtream_proxied_username
                    && credentials.last().unwrap().to_owned()
                        == xtream_config.xtream_proxied_password
                {
                    Ok(())
                } else {
                    Err(custom(Invalid {
                        status_code: StatusCode::FORBIDDEN,
                    }))
                }
            },
        )
        .untuple_one()
}
