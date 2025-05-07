// $coverage:ignore-start
//! Ignored from coverage because of real database interactions. covered as part of component tests
mod config;
pub mod env;

use crate::config::PGMonitorConfig;
use scylla_pg_core::config::PGConfig;
use scylla_pg_lib::manager::PgManager;
use std::time::Duration;

/// # Panics
/// In case invalid env config is passed.
pub async fn monitor_tasks() {
    let pgm = PgManager::from_config(&PGConfig::from_env().unwrap()).expect("Error creating PgManager Instance");
    let pg_monitor_config = PGMonitorConfig::from_env();
    loop {
        tokio::time::sleep(Duration::from_secs(pg_monitor_config.poll_interval)).await;
        reset_tasks(&pgm).await;
        match pgm.delete_terminated_tasks(pg_monitor_config.task_retention_time).await {
            Ok(count) => log::info!("tasks deleted: {count}"),
            Err(e) => log::error!("error occurred while deleting terminated tasks {e}"),
        };
    }
}

async fn reset_tasks(pgm: &PgManager) {
    match pgm.reset_batch().await {
        Ok(tasks) => {
            for task in tasks.iter() {
                log::debug!("task with {} has been reset to ready state ", task.rn);
            }
        }
        Err(e) => log::error!("error while resetting batch, {e:?}"),
    }
}
