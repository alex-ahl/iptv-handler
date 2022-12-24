use anyhow::{bail, Context};
use chrono::Utc;
use db::services::provider::{ExtInfApiModel, ProviderDBService};
use log::{error, info, trace};
use std::fmt::Write;
use tokio::fs::File;
use tokio::io::{AsyncWriteExt, BufWriter};
use url::Url;

pub async fn create_m3u_file(
    provider_service: ProviderDBService,
    proxy_domain: String,
) -> Result<(), anyhow::Error> {
    let path = build_file_path();

    if let Err(err) = compose_m3u(provider_service, &path, proxy_domain).await {
        error!("{}", err)
    }

    Ok(())
}

async fn compose_m3u(
    provider_service: ProviderDBService,
    path: &String,
    proxy_domain: String,
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

            if let Ok(line) = compose_extinf_lines(extinf, &proxy_domain) {
                writer
                    .write(line.as_bytes())
                    .await
                    .context("writing extinf line to file")?;
            }
        }

        writer.flush().await.context("Flushing output stream")?;

        let valid_extinf_entries = total_extinf_entries_length - extinf_excludes;

        info!("Excluded {} channels based on group", extinf_excludes);
        info!("Total extinf entries is {}", total_extinf_entries_length);
        info!("Wrote {} extinf entries to a .m3u", valid_extinf_entries)
    }

    Ok(())
}

fn build_file_path() -> String {
    let unix_timestamp = chrono::Local::now().timestamp();
    let timestamp = Utc::now().format("%Y-%m-%d_%H-%M-%S");
    let path = format!("{unix_timestamp}_{timestamp}.m3u");

    path
}

async fn create_file(path: &String) -> Result<File, anyhow::Error> {
    let file = File::create(path).await.context("creating .m3u file")?;

    Ok(file)
}

fn compose_extinf_lines(
    extinf: ExtInfApiModel,
    proxy_domain: &String,
) -> Result<String, anyhow::Error> {
    let mut line = String::new();

    write!(line, "#EXTINF:-1").context("writing #EXTINF:-1 line")?;

    if let None = extinf.attributes {
        bail!("No attributes..")
    }

    for attr in extinf.attributes.unwrap_or_default() {
        let attr_value = try_parse_url_from_attr(attr.value, attr.id, proxy_domain);

        write!(line, " {}=\"{}\"", attr.key, attr_value).context(format!(
            "writing attribute with key {} and value {} for extinf channel {}",
            attr.key, attr_value, extinf.name
        ))?;
    }

    let proxified_url = proxify_url(extinf.id, &proxy_domain, UrlType::Stream);

    write!(line, ",{}{}{}{}", extinf.name, "\n", proxified_url, "\n")
        .context(format!("writing extinf lines for channel {}", extinf.name))?;

    Ok(line)
}

fn proxify_url(id: u64, proxy_domain: &String, url_type: UrlType) -> String {
    let url_type = match url_type {
        UrlType::Stream => "stream".to_string(),
        UrlType::Attribute => "attr".to_string(),
    };

    format!("http://{}/{}/{}", proxy_domain, url_type, id)
}

fn try_parse_url_from_attr(val: String, id: u64, proxy_domain: &String) -> String {
    let url_parsed_attr = match Url::parse(&val) {
        Ok(res) => {
            if res.cannot_be_a_base() {
                return val;
            }

            proxify_url(id, proxy_domain, UrlType::Attribute)
        }
        Err(_) => Default::default(),
    };

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
