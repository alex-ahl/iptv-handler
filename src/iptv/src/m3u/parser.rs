use std::collections::HashMap;

use anyhow::Context;
use log::{debug, error, info, trace};
use regex::Regex;
use tokio::io::{AsyncBufReadExt, BufReader, Lines};
use url::Url;

use super::{
    fetcher::get_m3u,
    models::{ExtInf, M3U},
};

pub async fn parse_m3u_url(url: &Url) -> Result<M3U, anyhow::Error> {
    let m3u = get_m3u(&url).await.context("Could not get M3U content")?;
    let m3u_reader = BufReader::new(m3u.as_bytes()).lines();

    let extinfs = match process_lines(m3u_reader).await {
        Ok(extinfs) => extinfs,
        Err(err) => {
            error!("{}", err);
            vec![]
        }
    };

    Ok(M3U { extinfs })
}

async fn process_lines(mut lines: Lines<BufReader<&[u8]>>) -> Result<Vec<ExtInf>, anyhow::Error> {
    let mut total_line_count = 0;
    let mut invalid_line_count = 0;
    let mut invalid_extinf_entry_count = 0;

    let valid_extinf_line = Regex::new(r#"^(#\S+(?:\s+[^\s="]+=".*")+),(.*)\s*(.*)"#).unwrap();

    let mut parsed_extinf_entries: Vec<ExtInf> = vec![];

    while let Some(line) = lines.next_line().await? {
        total_line_count += 1;

        if !is_valid_line(&line) {
            invalid_line_count += 1;
            debug!("\nInvalid line ignored\n{}", line);
            continue;
        }

        if valid_extinf_line.is_match(&line) {
            let attributes = parse_attributes(&line);
            let group_title = get_group_title(&attributes);

            if let Some(url) = lines.next_line().await.context("Could not read line")? {
                if let Ok(url) = Url::parse(&url) {
                    parsed_extinf_entries.push(ExtInf {
                        name: parse_name(&line),
                        attributes,
                        url: url.clone(),
                        group_title,
                    });

                    trace!("\r\nSuccessfully parsed extinf\r\n{}\r\n{}", line, url);
                } else {
                    invalid_extinf_entry_count += 1;
                    invalid_line_count += 1;
                    debug!("\nSkipped invalid extinf entry\n{}\n{}", line.as_str(), url);
                    continue;
                };
            }
        }
    }

    log_lines_info(
        invalid_extinf_entry_count,
        invalid_line_count,
        total_line_count,
    );

    Ok(parsed_extinf_entries)
}

fn get_group_title(attributes: &HashMap<String, String>) -> String {
    attributes
        .get_key_value("group-title")
        .map(|value| value.1.clone())
        .unwrap_or_default()
}

fn is_valid_line(line: &String) -> bool {
    let valid_line_starters = vec!["#EXTINF", "#EXTM3U"];

    valid_line_starters
        .iter()
        .any(|valid_start| line.starts_with(valid_start))
}

fn parse_name(extinf_line: &str) -> String {
    extinf_line
        .split("\",")
        .last()
        .unwrap_or_default()
        .to_string()
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

fn log_lines_info(invalid_extinf_entry_count: i32, invalid_line_count: i32, total_line_count: i32) {
    info!(
        "Ignored {} invalid extinf entries",
        invalid_extinf_entry_count
    );

    info!(
        "Ignored {} invalid lines out of a total of {} lines",
        invalid_line_count, total_line_count
    );
}
