use std::{
    collections::HashMap,
    io::{BufRead, BufReader, Lines},
    iter::Skip,
};

use regex::Regex;
use url::Url;

use super::fetcher::get_m3u;

pub async fn parse_m3u(url: Url) {
    let res = get_m3u(&url).await.expect("getting of successful result");

    let m3u = BufReader::new(res.as_bytes()).lines();

    let mut lines = skip_ext_m3u_line(m3u);
    let valid_extinf_line = Regex::new(r#"^(#\S+(?:\s+[^\s="]+=".*")+),(.*)\s*(.*)"#).unwrap();

    while let (Some(metadata), Some(url)) = (lines.next(), lines.next()) {
        let metadata = metadata.unwrap();
        let url = Url::parse(&url.unwrap());

        if valid_extinf_line.is_match(&metadata) {
            let _attributes = parse_attributes(&metadata);
            let _url = url;
        } else {
            println!("{:?}", format!("{}{}", "nope: ", metadata));
        }
    }
}

fn skip_ext_m3u_line(lines: Lines<BufReader<&[u8]>>) -> Skip<Lines<BufReader<&[u8]>>> {
    lines.skip(1)
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
