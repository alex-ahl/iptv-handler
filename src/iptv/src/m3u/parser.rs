use std::{
    collections::HashMap,
    io::{BufRead, BufReader, Lines},
    iter::Skip,
    slice::Iter,
};

use anyhow::Context;
use log::{debug, error, info};
use regex::Regex;
use url::Url;

use super::{
    fetcher::get_m3u,
    models::{ExtInf, M3U},
};

pub async fn parse_m3u_url(url: &Url) -> Result<M3U, anyhow::Error> {
    let m3u = get_m3u(&url).await.context("Could not get M3U content")?;

    let m3u = BufReader::new(m3u.as_bytes()).lines();

    let m3u = pre_process(m3u);

    let lines = skip_ext_m3u_line(m3u.iter());

    let parsed_extinf_lines = parse_extinf_entries(lines).to_vec();

    Ok(M3U {
        extinfs: parsed_extinf_lines,
    })
}

fn pre_process(m3u: Lines<BufReader<&[u8]>>) -> Vec<String> {
    let array = vec!["#EXTINF", "http", "#EXTM3U"];

    let m3u = m3u.map(|line| line.unwrap()).collect::<Vec<String>>();

    let total_lines = m3u.len();

    let filtered_m3u = m3u
        .into_iter()
        .filter(|line| {
            let is_valid_line = array
                .iter()
                .any(|valid_start| line.starts_with(valid_start));

            if !is_valid_line {
                debug!("Invalid line removed: {}", line);
            }

            is_valid_line
        })
        .collect::<Vec<String>>();

    info!("Removed {} invalid lines", total_lines - filtered_m3u.len());

    filtered_m3u
}

fn skip_ext_m3u_line(lines: Iter<'_, String>) -> Skip<Iter<String>> {
    lines.skip(1)
}

fn parse_extinf_entries(lines: Skip<Iter<String>>) -> Box<Vec<ExtInf>> {
    let mut parsed_extinf_entries: Vec<ExtInf> = vec![];
    let valid_extinf_line = Regex::new(r#"^(#\S+(?:\s+[^\s="]+=".*")+),(.*)\s*(.*)"#).unwrap();

    let mut lines = lines.enumerate();

    while let (Some((.., metadata)), Some((.., url))) = (lines.next(), lines.next()) {
        let url = Url::parse(&url).context(format!("{}{}", "Could not parse EXTINF url", url));

        match url {
            Ok(url) if valid_extinf_line.is_match(&metadata) => {
                let attributes = parse_attributes(&metadata);
                let group_title = attributes
                    .get_key_value("group-title")
                    .map(|value| value.1.clone())
                    .unwrap_or_default();

                parsed_extinf_entries.push(ExtInf {
                    name: parse_name(&metadata),
                    attributes,
                    url: url.clone(),
                    group_title,
                });

                debug!(
                    "\r\nSuccessfully parsed line channel:\r\n{}\r\n{}",
                    metadata, url
                );
            }
            Ok(_) => error!("Could not parse line\r\n{}", metadata),
            Err(e) => error!("Could not parse URL\r\n{}", e),
        };
    }

    info!("Parsed {} extinf entries", parsed_extinf_entries.len());

    Box::new(parsed_extinf_entries)
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
