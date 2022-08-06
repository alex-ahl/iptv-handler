use anyhow::{bail, Context};
use chrono::Utc;
use db::services::provider::{ExtInfApiModel, ProviderApiModel};
use log::{error, info, trace};
use std::fmt::Write;
use tokio::fs::File;
use tokio::io::{AsyncWriteExt, BufWriter};

pub async fn create_m3u_file(
    provider: ProviderApiModel,
    group_excludes: Vec<String>,
) -> Result<(), anyhow::Error> {
    let path = build_file_path();

    if let Err(err) = compose_m3u(provider, &path, group_excludes).await {
        error!("{}", err)
    }

    Ok(())
}

async fn compose_m3u(
    provider: ProviderApiModel,
    path: &String,
    group_excludes: Vec<String>,
) -> Result<(), anyhow::Error> {
    let file = create_file(path).await?;
    let mut writer = BufWriter::new(file);

    writer
        .write("#EXTM3U\n".as_bytes())
        .await
        .context("writing #EXTM3U line to file")?;

    if let Some(extinfs) = provider.extinfs {
        let total_extinf_entries_length = extinfs.len();
        let mut extinf_excludes = 0;

        for extinf in extinfs {
            let should_exclude_extinf = check_group_exclusion(&extinf, &group_excludes);

            if should_exclude_extinf {
                extinf_excludes += 1;

                trace!("Excluded channel {} based on group filter", extinf.name);

                continue;
            }

            if let Ok(line) = compose_extinf_lines(extinf) {
                writer
                    .write(line.as_bytes())
                    .await
                    .context("writing extinf line to file")?;
            }
        }

        let valid_extinf_entries = total_extinf_entries_length - extinf_excludes;

        info!("Excluded {} channels based on group", extinf_excludes);
        info!("Total extinf entries is {}", total_extinf_entries_length);
        info!("Wrote {} extinf entries to a .m3u", valid_extinf_entries)
    }

    Ok(())
}

fn check_group_exclusion(extinf: &ExtInfApiModel, group_excludes: &Vec<String>) -> bool {
    let matched_group = extinf
        .attributes
        .as_ref()
        .unwrap()
        .iter()
        .map(|attr| (&attr.key, &attr.value))
        .find(|kvp| {
            kvp.0 == "group-title"
                && group_excludes
                    .iter()
                    .any(|exclude| kvp.1.to_ascii_lowercase().contains(&exclude.to_lowercase()))
        });

    match matched_group {
        Some(_) => true,
        None => false,
    }
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
        bail!("No attributes..")
    }

    for attr in extinf.attributes.unwrap_or_default() {
        write!(line, " {}=\"{}\"", attr.key, attr.value).context(format!(
            "writing attribute with key {} and value {} for extinf channel {}",
            attr.key, attr.value, extinf.name
        ))?;
    }

    write!(line, ", {}{}{}{}", extinf.name, "\n", extinf.url, "\n")
        .context(format!("writing extinf lines for channel {}", extinf.name))?;

    Ok(line)
}
