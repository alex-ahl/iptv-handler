use std::{
    collections::HashMap,
    io::{BufRead, BufReader, Lines},
    iter::Skip,
};

use anyhow::Context;
use log::{info, warn};
use regex::Regex;
use url::Url;

use super::{
    fetcher::get_m3u,
    models::{ExtInf, M3U},
};

pub async fn parse_m3u_url(url: Url) -> Result<M3U, anyhow::Error> {
    let res = get_m3u(&url).await.context("Could not get M3U content")?;

    let m3u = BufReader::new(res.as_bytes()).lines();

    // IS VALID M3U file here

    let lines = skip_ext_m3u_line(m3u);

    let parsed_extinf_lines = parse_extinf_lines(lines).to_vec();

    Ok(M3U {
        extinfs: parsed_extinf_lines,
    })
}

fn skip_ext_m3u_line(lines: Lines<BufReader<&[u8]>>) -> Skip<Lines<BufReader<&[u8]>>> {
    lines.skip(1)
}

fn parse_extinf_lines(mut lines: Skip<Lines<BufReader<&[u8]>>>) -> Box<Vec<ExtInf>> {
    let mut parsed_extinf_lines: Vec<ExtInf> = vec![];
    let valid_extinf_line = Regex::new(r#"^(#\S+(?:\s+[^\s="]+=".*")+),(.*)\s*(.*)"#).unwrap();

    while let (Some(metadata), Some(url)) = (lines.next(), lines.next()) {
        let metadata = metadata.unwrap();
        let url = Url::parse(&url.unwrap());

        if valid_extinf_line.is_match(&metadata) {
            parsed_extinf_lines.push(ExtInf {
                name: parse_name(&metadata),
                attributes: parse_attributes(&metadata),
                url: url.unwrap(),
            });

            info!("Successfully parsed line: {}", metadata);
        } else {
            warn!("Could not parse line: {}", metadata)
        }
    }

    Box::new(parsed_extinf_lines)
}

fn parse_name(extinf_line: &str) -> String {
    extinf_line.split("\",").last().unwrap().to_string()
}

fn parse_attributes(extinf_line: &str) -> HashMap<String, String> {
    let key_value_pairs = Regex::new(r#"[^\s"]+(?:"[^"]*")"#).unwrap();

    let attributes: HashMap<_, _> = key_value_pairs
        .captures_iter(extinf_line)
        .map(|cap| {
            let key = cap[0].split("=").next().unwrap().to_owned();
            let value = cap[0].split("\"").nth(1).unwrap().to_owned();

            (key, value)
        })
        .collect();

    attributes
}
