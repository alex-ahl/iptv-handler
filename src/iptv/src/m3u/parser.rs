use std::{collections::HashMap, str::Split, sync::Arc};

use anyhow::{Context, Error};
use db::{models::GroupRequest, services::provider::ExtInf};
use log::{debug, error, info, trace};
use regex::Regex;
use rest_client::RestClient;
use tokio::{
    io::{AsyncBufReadExt, BufReader, Lines},
    try_join,
};
use url::Url;

use crate::models::{ParsedM3u, XtreamCategory, XtreamConfig};

use super::fetcher::get_m3u;

pub async fn parse_m3u_url(
    url: &Url,
    group_excludes: &Vec<String>,
    xtream_config: XtreamConfig,
    client: Arc<RestClient>,
) -> Result<ParsedM3u, Error> {
    let m3u = get_m3u(&url).await.context("Could not get M3U content")?;
    let m3u_reader = BufReader::new(m3u.as_bytes()).lines();

    let m3u = match process_lines(m3u_reader, group_excludes, xtream_config, client).await {
        Ok(extinfs) => extinfs,
        Err(err) => {
            error!("{}", err);
            ParsedM3u::default()
        }
    };

    Ok(m3u)
}

async fn process_lines(
    mut lines: Lines<BufReader<&[u8]>>,
    group_excludes: &Vec<String>,
    xtream_config: XtreamConfig,
    client: Arc<RestClient>,
) -> Result<ParsedM3u, Error> {
    let mut total_line_count = 0;
    let mut invalid_line_count = 0;
    let mut invalid_extinf_entry_count = 0;

    let valid_extinf_line = Regex::new(r#"^(#\S+(?:\s+[^\s="]+=".*")+),(.*)\s*(.*)"#).unwrap();

    let mut parsed_extinf_entries: Vec<ExtInf> = vec![];
    let mut groups: Vec<GroupRequest> = vec![];

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

            let exclude = should_be_excluded(&group_title, &group_excludes);

            if let Some(url) = lines.next_line().await.context("Could not read line")? {
                if let Ok(url) = Url::parse(&url) {
                    let path_segments = get_path_segments(&url);
                    let last_segment = get_last_path_segment(&path_segments);

                    parsed_extinf_entries.push(ExtInf {
                        name: parse_name(&line),
                        attributes,
                        url: url.clone(),
                        track_id: parse_track_id(&last_segment),
                        prefix: parse_prefix(&path_segments),
                        extension: parse_extension(last_segment),
                        group_title: group_title.clone(),
                        exclude,
                    });

                    if !is_duplicate_group(&groups, group_title.clone()) {
                        groups.push(GroupRequest {
                            name: group_title,
                            exclude,
                            xtream_cat_id: None,
                            m3u_id: None,
                        });
                    }

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

    if xtream_config.enabled {
        groups = try_set_category_ids(groups, xtream_config, client).await?;
    }

    log_lines_info(
        invalid_extinf_entry_count,
        invalid_line_count,
        total_line_count,
    );

    let res = ParsedM3u {
        extinfs: parsed_extinf_entries,
        groups,
    };

    Ok(res)
}

async fn try_set_category_ids(
    mut groups: Vec<GroupRequest>,
    xtream_config: XtreamConfig,
    client: Arc<RestClient>,
) -> Result<Vec<GroupRequest>, Error> {
    let xtream_categories = try_get_xtream_categories(&xtream_config, &client).await?;

    for group in &mut groups {
        let cat = xtream_categories
            .clone()
            .into_iter()
            .find(|cat| cat.category_name == group.name);

        if let Some(cat) = cat {
            group.xtream_cat_id = Some(cat.category_id.parse().unwrap_or_default());
        }
    }

    Ok(groups)
}

fn is_duplicate_group(groups: &Vec<GroupRequest>, group_title: String) -> bool {
    if groups
        .clone()
        .into_iter()
        .any(|group| group.name == group_title)
    {
        true
    } else {
        false
    }
}

fn get_path_segments(url: &Url) -> Split<char> {
    url.path_segments().ok_or("no segments").unwrap()
}

fn get_last_path_segment(segments: &Split<char>) -> String {
    segments
        .clone()
        .last()
        .ok_or("no items")
        .map(String::from)
        .unwrap_or_default()
}

fn parse_prefix(segments: &Split<char>) -> Option<String> {
    let first_segment = segments.clone().next().unwrap_or_default();

    let valid_prefix = vec!["live", "movie", "series"];

    let is_valid_prefix = valid_prefix
        .iter()
        .any(|valid_start| first_segment.eq(*valid_start));

    if !is_valid_prefix {
        return Some(String::new());
    }

    Some(String::from(first_segment))
}

pub fn parse_track_id(last_segment: &String) -> Option<String> {
    if last_segment.is_empty() {
        return None;
    }

    if last_segment.contains(".") {
        return Some(last_segment.split('.').nth(0).unwrap().to_string());
    }

    Some(last_segment.clone())
}

pub fn parse_extension(last_segment: String) -> Option<String> {
    if last_segment.is_empty() {
        return None;
    }

    if !last_segment.contains(".") {
        return None;
    }

    Some(last_segment.split('.').nth(1).unwrap().to_string())
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

fn should_be_excluded(group_title: &String, group_excludes: &Vec<String>) -> bool {
    if group_title.is_empty() {
        return false;
    }

    let exclude = group_excludes.iter().any(|exclude| {
        (*group_title)
            .to_ascii_lowercase()
            .contains(&exclude.to_lowercase())
    });

    exclude
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

async fn try_get_xtream_categories(
    xtream_config: &XtreamConfig,
    client: &Arc<RestClient>,
) -> Result<Vec<XtreamCategory>, Error> {
    let live_categories = get_xtream_category("get_live_categories", &xtream_config, &client);
    let vod_categories = get_xtream_category("get_vod_categories", &xtream_config, &client);
    let series_categories = get_xtream_category("get_series_categories", &xtream_config, &client);

    let (live, vod, series) = try_join!(live_categories, vod_categories, series_categories)?;

    let categories = live
        .into_iter()
        .chain(vod.into_iter().chain(series.into_iter()))
        .collect::<Vec<XtreamCategory>>();

    Ok(categories)
}

async fn get_xtream_category(
    action: &str,
    xtream_config: &XtreamConfig,
    client: &Arc<RestClient>,
) -> Result<Vec<XtreamCategory>, Error> {
    let xtream_categories = client
        .get(&Url::parse(
            format!(
                "http://{}/player_api.php?username={}&password={}&action={}",
                xtream_config.base_domain, xtream_config.username, xtream_config.password, action
            )
            .as_str(),
        )?)
        .await
        .context("getting xtream categories for parser")?
        .json::<Vec<XtreamCategory>>()
        .await
        .context("deserializing xtream categoris vector in parser")?;

    Ok(xtream_categories)
}
