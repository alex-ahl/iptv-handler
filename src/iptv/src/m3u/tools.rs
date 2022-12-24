use db::services::provider::M3U;
use itertools::Itertools;
use std::convert::TryInto;

pub fn count_groups(m3u: &M3U) -> u32 {
    m3u.extinfs
        .iter()
        .unique_by(|extinf| &extinf.group_title)
        .filter(|extinf| !extinf.group_title.is_empty())
        .count()
        .try_into()
        .unwrap_or_default()
}

pub fn count_channels(m3u: &M3U) -> u32 {
    m3u.extinfs.iter().count().try_into().unwrap_or_default()
}
