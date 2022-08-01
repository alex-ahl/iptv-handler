use std::sync::Arc;

use chrono::Duration;
use db::DB;
use log::{debug, error};
use tokio_cron_scheduler::{Job, JobScheduler};

use crate::{app::try_provider_update, environment::Configuration};

pub(crate) fn init_jobs(config: Configuration, db: Arc<DB>) {
    let schedule = JobScheduler::new().unwrap();

    let update_provider_job = create_update_provider_job(config, db);

    schedule
        .add(update_provider_job)
        .expect("Could not add scheduled job");

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
    .expect("Could not schedule job");

    update_provider_job
}
