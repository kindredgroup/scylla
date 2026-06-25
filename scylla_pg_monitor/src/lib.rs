// $coverage:ignore-start
//! Ignored from coverage because of real database interactions. covered as part of component tests
mod config;
pub mod env;
mod metrics;

use crate::config::PGMonitorConfig;
use crate::metrics::{init_otel_metrics, MetricsState};
use scylla_pg_core::config::PGConfig;
use scylla_pg_lib::manager::PgManager;
use std::{io, sync::Arc, time::Duration};

/// # Panics
/// In case invalid env config is passed.
pub async fn monitor_tasks() {
    let pg_monitor_config = PGMonitorConfig::from_env();
    let pgm = Arc::new(PgManager::from_config(&PGConfig::from_env().unwrap()).expect("Error creating PgManager Instance"));
    init_otel_metrics(pg_monitor_config.otel_grpc_endpoint.clone()).expect("Unable to initialise OTEL metrics exporter");
    let metrics_state = MetricsState::default();

    if let Err(e) = tokio::try_join!(
        run_cleanup_loop(Arc::clone(&pgm), pg_monitor_config.poll_interval, pg_monitor_config.task_retention_time),
        refresh_task_count_metrics_loop(Arc::clone(&pgm), metrics_state, pg_monitor_config.metrics_refresh_interval)
    ) {
        panic!("monitor runtime failed: {e}");
    }
}

async fn run_cleanup_loop(pgm: Arc<PgManager>, poll_interval: u64, task_retention_time: i64) -> io::Result<()> {
    loop {
        tokio::time::sleep(Duration::from_secs(poll_interval)).await;
        reset_tasks(&pgm).await;
        match pgm.delete_terminated_tasks(task_retention_time).await {
            Ok(count) => log::info!("tasks deleted: {count}"),
            Err(e) => log::error!("error occurred while deleting terminated tasks {e}"),
        };
    }
}

async fn refresh_task_count_metrics_loop(pgm: Arc<PgManager>, metrics_state: MetricsState, refresh_interval: u64) -> io::Result<()> {
    let refresh_interval = refresh_interval.max(1);
    loop {
        match pgm.fetch_task_counts_by_status().await {
            Ok(counts) => metrics_state.update_task_counts(counts).await,
            Err(e) => log::error!("error occurred while reading task counts by status {e}"),
        }
        tokio::time::sleep(Duration::from_secs(refresh_interval)).await;
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
