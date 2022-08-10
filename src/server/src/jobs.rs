use std::{ffi::OsStr, path::Path, sync::Arc};

use chrono::Duration;
use db::DB;
use log::{debug, error, info};
use tokio::fs;
use tokio_cron_scheduler::{Job, JobScheduler};

use crate::{app::try_provider_update, environment::Configuration};

pub(crate) fn init_jobs(config: Configuration, db: Arc<DB>) {
    let schedule = JobScheduler::new().unwrap();

    let update_provider_job = create_update_provider_job(config, db);
    let remove_obsolete_m3u_files = create_remove_obsolete_m3u_files_job();

    schedule
        .add(update_provider_job)
        .expect("Could not add update provider job");

    schedule
        .add(remove_obsolete_m3u_files)
        .expect("Could not add obsolete m3u files job");

    if let Err(e) = schedule.start() {
        error!("Error on scheduler {:?}", e);
    }
}

fn create_update_provider_job(config: Configuration, db: Arc<DB>) -> Job {
    let update_provider_job = Job::new_repeated_async(
        Duration::hours(config.hourly_update_frequency.into())
            .to_std()
            .unwrap(),
        move |_uuid, _l| {
            let db = db.clone();
            let config = config.clone();

            Box::pin(async move {
                debug!("Running provider update job");
                try_provider_update(config, db).await;
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

                files.sort();

                let number_of_files = files.len();

                if (number_of_files > 2) {
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

                    info!("Deleted {} obsolete m3u files", number_of_files_to_delete);
                } else {
                    info!("No m3u files eligible for removal",);
                }
            })
        })
        .expect("Could not schedule delete obsolete m3u files job");

    remove_obsolete_m3u_files
}
