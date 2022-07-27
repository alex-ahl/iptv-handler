use anyhow::{bail, Context};
use chrono::Utc;
use db::services::provider::{ExtInfApiModel, ProviderApiModel};
use log::{error, info};
use std::fmt::Write;
use tokio::fs::File;
use tokio::io::{AsyncWriteExt, BufWriter};

pub async fn create_m3u_file(provider: ProviderApiModel) -> Result<(), anyhow::Error> {
    let path = build_file_path();

    if let Err(err) = compose_m3u(provider, &path).await {
        error!("{}", err)
    }

    Ok(())
}

async fn compose_m3u(provider: ProviderApiModel, path: &String) -> Result<(), anyhow::Error> {
    let file = create_file(path).await?;
    let mut writer = BufWriter::new(file);

    writer
        .write("#EXTM3U\n".as_bytes())
        .await
        .context("writing #EXTM3U line to file")?;

    if let Some(extinfs) = provider.extinfs {
        let extinf_entries_length = extinfs.len();

        for extinf in extinfs {
            if let Ok(line) = compose_extinf_lines(extinf) {
                writer
                    .write(line.as_bytes())
                    .await
                    .context("writing extinf line to file")?;
            }
        }

        info!(
            "Wrote {} extinf entries to .m3u file",
            extinf_entries_length
        );
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

fn compose_extinf_lines(extinf: ExtInfApiModel) -> Result<String, anyhow::Error> {
    let mut line = String::new();

    write!(line, "#EXTINF:-1").context("writing #EXTINF:-1 line")?;

    if let None = extinf.attributes {
        bail!("")
    }

    for attr in extinf.attributes.unwrap() {
        write!(line, " {}=\"{}\"", attr.key, attr.value).context(format!(
            "writing attribute with key {} and value {} for extinf channel {}",
            attr.key, attr.value, extinf.name
        ))?;
    }

    write!(line, ", {}{}{}{}", extinf.name, "\n", extinf.url, "\n")
        .context(format!("writing extinf lines for channel {}", extinf.name))?;

    Ok(line)
}
