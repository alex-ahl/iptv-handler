use anyhow::{bail, Context, Error};
use chrono::Utc;
use db::services::provider::{ExtInfApiModel, ProviderDBService};
use log::{error, info, trace};
use std::fmt::Write;
use tokio::fs::File;
use tokio::io::{AsyncWriteExt, BufWriter};
use tokio::task::JoinHandle;
use tokio::{spawn, try_join};
use url::Url;

use crate::models::{IptvConfiguration, M3uType};

pub async fn create_m3u_file(
    service: ProviderDBService,
    iptv_config: IptvConfiguration,
) -> Result<(), Error> {
    let custom_handle = compose_custom_m3u(service.clone(), iptv_config.clone());
    let ts_handle = compose_ts_m3u(service.clone(), iptv_config.clone());
    let m3u8_handle = compose_m3u8_m3u(service.clone(), iptv_config.clone());

    match try_join!(custom_handle, ts_handle, m3u8_handle) {
        Ok(_) => (),
        Err(err) => {
            error!("{}", err);
        }
    }

    Ok(())
}

async fn compose_custom_m3u(
    provider_db_service: ProviderDBService,
    iptv_config: IptvConfiguration,
) -> Result<JoinHandle<()>, Error> {
    let custom_path = build_file_path(M3uType::Custom);

    let custom_handle = spawn(async move {
        if let Err(err) = compose_m3u(
            provider_db_service,
            &custom_path,
            iptv_config,
            M3uType::Custom,
            false,
        )
        .await
        {
            error!("{}", err)
        }
    });

    Ok(custom_handle)
}

async fn compose_ts_m3u(
    provider_db_service: ProviderDBService,
    iptv_config: IptvConfiguration,
) -> Result<JoinHandle<()>, Error> {
    let custom_path = build_file_path(M3uType::Ts);

    let custom_handle = spawn(async move {
        if let Err(err) = compose_m3u(
            provider_db_service,
            &custom_path,
            iptv_config,
            M3uType::Ts,
            false,
        )
        .await
        {
            error!("{}", err)
        }
    });

    Ok(custom_handle)
}

async fn compose_m3u8_m3u(
    provider_db_service: ProviderDBService,
    iptv_config: IptvConfiguration,
) -> Result<JoinHandle<()>, Error> {
    let custom_path = build_file_path(M3uType::M3u8);

    let custom_handle = spawn(async move {
        if let Err(err) = compose_m3u(
            provider_db_service,
            &custom_path,
            iptv_config,
            M3uType::M3u8,
            true,
        )
        .await
        {
            error!("{}", err)
        }
    });

    Ok(custom_handle)
}

async fn compose_m3u(
    provider_service: ProviderDBService,
    path: &String,
    iptv_config: IptvConfiguration,
    m3u_type: M3uType,
    log: bool,
) -> Result<(), anyhow::Error> {
    let file = create_file(path).await?;
    let mut writer = BufWriter::new(file);

    writer
        .write("#EXTM3U\n".as_bytes())
        .await
        .context("writing #EXTM3U line to file")?;

    if let Some(extinfs) = provider_service.extinfs {
        let total_extinf_entries_length = extinfs.len();
        let mut extinf_excludes = 0;

        for extinf in extinfs {
            if extinf.exclude {
                extinf_excludes += 1;

                trace!("Excluded channel {} based on group filter", extinf.name);

                continue;
            }

            if let Ok(line) = compose_extinf_lines(extinf, iptv_config.clone(), m3u_type) {
                writer
                    .write(line.as_bytes())
                    .await
                    .context("writing extinf line to file")?;
            }
        }

        writer.flush().await.context("Flushing output stream")?;

        let valid_extinf_entries = total_extinf_entries_length - extinf_excludes;

        if log {
            info!("Excluded {} channels based on group", extinf_excludes);
            info!("Total extinf entries is {}", total_extinf_entries_length);
            info!(
                "Wrote {} extinf entries to multiple .m3u files (ts, m3u8, custom)",
                valid_extinf_entries
            )
        }
    }

    Ok(())
}

fn build_file_path(m3u_type: M3uType) -> String {
    let unix_timestamp = chrono::Local::now().timestamp();
    let timestamp = Utc::now().format("%Y-%m-%d_%H-%M-%S");

    let path = match m3u_type {
        M3uType::M3u8 => format!("m3u8_{unix_timestamp}_{timestamp}.m3u"),
        M3uType::Ts => format!("ts_{unix_timestamp}_{timestamp}.m3u"),
        M3uType::Custom => format!("custom_{unix_timestamp}_{timestamp}.m3u"),
    };

    path
}

async fn create_file(path: &String) -> Result<File, anyhow::Error> {
    let file = File::create(path).await.context("creating .m3u file")?;

    Ok(file)
}

fn compose_extinf_lines(
    extinf: ExtInfApiModel,
    iptv_config: IptvConfiguration,
    m3u_type: M3uType,
) -> Result<String, Error> {
    let mut line = String::new();

    write!(line, "#EXTINF:-1").context("writing #EXTINF:-1 line")?;

    if let None = extinf.attributes {
        bail!("No attributes..")
    }

    for attr in extinf.attributes.unwrap_or_default() {
        let attr_value = try_parse_url_from_attr(attr.value, attr.id, iptv_config.clone());

        write!(line, " {}=\"{}\"", attr.key, attr_value).context(format!(
            "writing attribute with key {} and value {} for extinf channel {}",
            attr.key, attr_value, extinf.name
        ))?;
    }

    let proxified_url = match m3u_type {
        M3uType::Custom => proxify_url(
            extinf.id.to_string(),
            extinf.prefix,
            extinf.extension,
            iptv_config,
            UrlType::Stream,
            Some(m3u_type),
        ),
        _ => proxify_url(
            extinf.track_id,
            extinf.prefix,
            extinf.extension,
            iptv_config,
            UrlType::Stream,
            Some(m3u_type),
        ),
    }?;

    write!(line, ",{}{}{}{}", extinf.name, "\n", proxified_url, "\n")
        .context(format!("writing extinf lines for channel {}", extinf.name))?;

    Ok(line)
}

fn proxify_url(
    id: String,
    prefix: Option<String>,
    extension: Option<String>,
    iptv_config: IptvConfiguration,
    url_type: UrlType,
    m3u_type: Option<M3uType>,
) -> Result<String, anyhow::Error> {
    let mut url = String::new();

    write!(url, "http://{}", iptv_config.proxy_domain).unwrap_or_default();

    match url_type {
        UrlType::Stream => {
            match m3u_type.unwrap_or_default() {
                M3uType::Custom => {
                    write!(url, "/{}/{}", "stream", id)?;
                }
                M3uType::Ts => {
                    if let Some(prefix) = prefix {
                        if prefix != "live" && !prefix.is_empty() {
                            write!(url, "/{}", prefix)?;
                        }
                    }

                    write!(
                        url,
                        "/{}/{}/{}",
                        iptv_config.xtream_username, iptv_config.xtream_password, id
                    )?;

                    if let Some(extension) = extension {
                        if extension != "m3u8" {
                            write!(url, ".{}", extension)?;
                        }
                    }
                }
                M3uType::M3u8 => {
                    if let Some(prefix) = prefix {
                        if prefix.is_empty() {
                            write!(url, "/{}", "live")?;
                        } else {
                            write!(url, "/{}", prefix)?;
                        }
                    }

                    write!(
                        url,
                        "/{}/{}/{}",
                        iptv_config.xtream_username, iptv_config.xtream_password, id
                    )?;

                    if let Some(extension) = extension {
                        write!(url, ".{}", extension)?;
                    } else {
                        write!(url, ".{}", "m3u8")?;
                    }
                }
            };
        }
        UrlType::Attribute => {
            write!(url, "/{}/{}", "attr", id)?;
        }
    };

    Ok(url)
}

fn try_parse_url_from_attr(val: String, id: u64, iptv_config: IptvConfiguration) -> String {
    let url_parsed_attr = match Url::parse(&val) {
        Ok(res) => {
            if res.cannot_be_a_base() {
                return val;
            }

            proxify_url(
                id.to_string(),
                None,
                None,
                iptv_config,
                UrlType::Attribute,
                None,
            )
        }
        Err(_) => Ok(Default::default()),
    }
    .unwrap_or_default();

    let attr_value = if url_parsed_attr.is_empty() {
        val
    } else {
        url_parsed_attr
    };

    attr_value
}

#[derive(PartialEq)]
enum UrlType {
    Stream,
    Attribute,
}
