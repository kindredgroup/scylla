use scylla_pg_core::config::PGConfig;
use scylla_pg_lib::manager::PgManager;
use std::env;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tokio::sync::Semaphore;

#[tokio::main]
pub async fn main() {
    env_logger::builder().format_timestamp_millis().init();
    let pgm = PgManager::from_config(&PGConfig::from_env().unwrap()).expect("Error creating PgManager Instance");
    let args: Vec<String> = env::args().collect();
    let worker = &args[1];
    log::info!("worker: {}", worker);
    let semaphore = Arc::new(Semaphore::new(5));

    let pgm_rw: Arc<RwLock<PgManager>> = Arc::new(RwLock::new(pgm));

    loop {
        tokio::time::sleep(Duration::from_millis(5)).await;
        let pgm_clone = pgm_rw.clone();
        let worker_clone = worker.clone();
        let permit = semaphore.clone().acquire_owned().await.unwrap();
        tokio::spawn(async move {
            match pgm_clone.read().await.lease_n_tasks("load_test".to_string(), 10, worker_clone, Some(5)).await {
                Err(e) => {
                    log::error!("error occurred while leasing tasks {e}");
                }
                Ok(tasks) => {
                    for t in tasks {
                        if let Err(e) = pgm_clone.read().await.heartbeat_task(t.rn.clone(), t.owner.unwrap(), None, None).await {
                            log::error!("error occurred while heartbeat tasks {e}");
                        }
                        if let Err(e) = pgm_clone.read().await.complete_task(t.rn.clone()).await {
                            log::error!("error occurred while complete tasks {e}");
                        }
                    }
                }
            }
            drop(permit);
        });
    }
}
