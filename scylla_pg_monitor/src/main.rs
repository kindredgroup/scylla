// $coverage:ignore-start
use scylla_pg_monitor::monitor_tasks;
#[tokio::main]
async fn main() {
  monitor_tasks().await;
}
