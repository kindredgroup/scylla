// $coverage:ignore-start
use scylla_pg_monitor::monitor_tasks;
#[tokio::main]
async fn main() {
    env_logger::builder().format_timestamp_millis().init();
    monitor_tasks().await;
}
