use std::{
    convert::Infallible,
    ffi::OsStr,
    path::{Path, PathBuf},
};

use log::debug;
use tokio::fs::{read_dir, DirEntry, File};
use tokio_util::io::ReaderStream;

use warp::{hyper::Body, reply::Response};

pub async fn get_latest_m3u_file() -> Result<Response, Infallible> {
    let path = get_latest_m3u_path().await;

    let response = match File::open(path).await {
        Ok(file) => {
            let stream = ReaderStream::new(file);

            let body = Body::wrap_stream(stream);

            let response = warp::hyper::Response::builder()
                .status(200)
                .header(
                    "Content-Disposition",
                    "attachement; filename = \"playlist.m3u\"",
                )
                .body(body)
                .unwrap();

            response
        }
        Err(_) => {
            debug!("No m3u file available");
            warp::hyper::Response::builder()
                .status(200)
                .body(Body::default())
                .unwrap()
        }
    };

    Ok(response)
}

async fn get_latest_m3u_path() -> PathBuf {
    let mut dir = read_dir(".").await.unwrap();

    let mut files: Vec<DirEntry> = vec![];

    while let Some(file) = dir.next_entry().await.unwrap_or_default() {
        let extension = file
            .path()
            .extension()
            .and_then(OsStr::to_str)
            .unwrap_or_default()
            .to_owned();

        if extension == "m3u" {
            files.push(file)
        }
    }

    let mut paths: Vec<PathBuf> = files.iter().map(|file| file.path()).collect();
    paths.sort();

    let freshesh_file = match paths.last() {
        Some(path) => path.to_path_buf(),
        None => PathBuf::new(),
    };

    freshesh_file
}

pub async fn serve_file_by_file_name(file_name: String) -> Result<Response, Infallible> {
    let path = Path::new(file_name.as_str());
    let file = tokio::fs::File::open(path)
        .await
        .expect("Could not open file from disc");

    let stream = ReaderStream::new(file);
    let body = Body::wrap_stream(stream);

    let response = warp::hyper::Response::builder()
        .status(200)
        .header(
            "Content-Disposition",
            "attachement; filename = \"playlist.m3u\"",
        )
        .body(body)
        .unwrap_or_default();

    Ok(response)
}

pub async fn m3u_file_exist() -> Result<Response, Infallible> {
    let path = get_latest_m3u_path().await;

    let res = match File::open(path).await {
        Ok(_) => warp::hyper::Response::builder()
            .status(200)
            .body(Body::default())
            .unwrap_or_default(),
        Err(_) => warp::hyper::Response::builder()
            .status(403)
            .body(Body::default())
            .unwrap_or_default(),
    };

    Ok(res)
}
