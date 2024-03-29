use std::{ffi::OsStr, path::Path, sync::Arc};

use api::{
    handlers::provider::{delete_provider, get_provider_entries_by_url},
    models::ApiConfiguration,
};
use chrono::Duration;
use db::{models::ProviderModel, DB};
use iptv::models::IptvConfiguration;
use log::{debug, error, info};
use rest_client::RestClient;
use tokio::fs;
use tokio_cron_scheduler::{Job, JobScheduler};

use crate::{app::try_provider_update, environment::Configuration, tools::deserialize_body};

pub(crate) fn init_jobs(
    config: Configuration,
    api_config: ApiConfiguration,
    iptv_config: IptvConfiguration,
    db: Arc<DB>,
    client: Arc<RestClient>,
) {
    let schedule = JobScheduler::new().unwrap();

    let update_provider_job = create_update_provider_job(
        config.clone(),
        api_config,
        iptv_config,
        db.clone(),
        client.clone(),
    );
    let remove_obsolete_m3u_files = create_remove_obsolete_m3u_files_job();
    let purge_obsolete_provider_entries =
        create_purge_obsolete_provider_entries(config, db, client);

    schedule
        .add(update_provider_job)
        .expect("Could not add update provider job");

    schedule
        .add(remove_obsolete_m3u_files)
        .expect("Could not add obsolete m3u files job");

    schedule
        .add(purge_obsolete_provider_entries)
        .expect("Could not add purge obsolete provider entries");

    if let Err(e) = schedule.start() {
        error!("Error on scheduler {:?}", e);
    }
}

fn create_update_provider_job(
    config: Configuration,
    api_config: ApiConfiguration,
    iptv_config: IptvConfiguration,
    db: Arc<DB>,
    client: Arc<RestClient>,
) -> Job {
    let update_provider_job = Job::new_repeated_async(
        Duration::hours(config.hourly_update_frequency.into())
            .to_std()
            .unwrap(),
        move |_uuid, _l| {
            let db = db.clone();
            let client = client.clone();
            let config = config.clone();
            let api_config = api_config.clone();
            let iptv_config = iptv_config.clone();

            Box::pin(async move {
                debug!("Running provider update job");
                try_provider_update(config, api_config, iptv_config, db, client).await;
            })
        },
    )
    .expect("Could not schedule update provider job");

    update_provider_job
}

fn create_remove_obsolete_m3u_files_job() -> Job {
    let remove_obsolete_m3u_files =
        Job::new_repeated_async(Duration::hours(6).to_std().unwrap(), move |_uuid, _l| {
            Box::pin(async move {
                debug!("Running delete obsolete m3u files job");

                let mut dir = tokio::fs::read_dir(".").await.unwrap();

                let mut files: Vec<String> = vec![];

                while let Some(file) = dir.next_entry().await.unwrap_or_default() {
                    let extension = file
                        .path()
                        .extension()
                        .and_then(OsStr::to_str)
                        .unwrap_or_default()
                        .to_owned();

                    if extension == "m3u" {
                        files.push(file.file_name().into_string().unwrap())
                    }
                }

                let file_starts: Vec<&str> = vec!["custom", "ts", "m3u8"];

                for starts_with in file_starts {
                    let mut files: Vec<String> = files
                        .clone()
                        .iter()
                        .filter(|file| file.starts_with(starts_with))
                        .map(|file| file.to_string())
                        .collect();

                    files.sort();

                    let number_of_files = files.len();

                    if number_of_files > 2 {
                        let number_of_files_to_delete = number_of_files - (number_of_files - 2);

                        let files_to_delete = files
                            .into_iter()
                            .take(number_of_files_to_delete)
                            .collect::<Vec<String>>();

                        for file_name in files_to_delete {
                            let path = Path::new(&file_name);
                            fs::remove_file(path).await.unwrap_or_default();
                            debug!("Removed file {}", file_name);
                        }

                        info!(
                            "Deleted {} obsolete {} m3u files",
                            number_of_files_to_delete, starts_with
                        );
                    } else {
                        info!("No {} m3u files eligible for removal", starts_with);
                    }
                }
            })
        })
        .expect("Could not schedule delete obsolete m3u files job");

    remove_obsolete_m3u_files
}

fn create_purge_obsolete_provider_entries(
    config: Configuration,
    db: Arc<DB>,
    client: Arc<RestClient>,
) -> Job {
    let purge_obsolete_provider_entries =
        Job::new_repeated_async(Duration::hours(24).to_std().unwrap(), move |_uuid, _l| {
            let db = db.clone();
            let client = client.clone();

            let config = config.clone();

            Box::pin(async move {
                debug!("Running purge obsolete provider entries");

                let response = get_provider_entries_by_url(config.m3u.as_str(), db.clone(), client)
                    .await
                    .expect("Could not get provider created date");

                let mut provider = deserialize_body::<Vec<ProviderModel>>(response)
                    .await
                    .unwrap_or_default();

                provider.sort_by_key(|x| x.created_at);

                let obsolete_provider_entries = provider.split_last().unwrap().1;

                for provider in obsolete_provider_entries {
                    let _res = delete_provider(provider.id, db.clone())
                        .await
                        .unwrap_or_default();
                }
            })
        })
        .expect("Could not schedule purge obsolete provider entries");

    purge_obsolete_provider_entries
}
