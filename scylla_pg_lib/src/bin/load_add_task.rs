use std::sync::Arc;
use std::time::Duration;
use serde_json::{json, Value};
use uuid::{Uuid};
use log::log;
use tokio::sync::RwLock;
use scylla_models::AddTaskModel;
use scylla_pg_core::config::PGConfig;
use scylla_pg_lib::manager::PgManager;

#[tokio::main]
pub async fn main() {
    env_logger::builder().format_timestamp_millis().init();
    let pgm = PgManager::from_config(&PGConfig::from_env().unwrap()).expect("Error creating PgManager Instance");
    let pgm_rw: Arc<RwLock<PgManager>> = Arc::new(RwLock::new(pgm));
    loop {
        let pgm_clone = pgm_rw.clone();
        tokio::time::sleep(Duration::from_millis(5)).await;
        tokio::spawn(async move {
            let atm = AddTaskModel {
                rn: String::from(Uuid::new_v4()),
                spec: json!("{}"),
                priority: 0,
                queue: "load_test".to_string(),
            };
            if let Err(e) = pgm_clone.read().await.insert_task(atm).await {
                log::error!("error occurred while adding tasks {e}")
            }
        });
    }
}