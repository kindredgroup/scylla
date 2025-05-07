use scylla_pg_core::config::PGConfig;
use scylla_pg_lib::analyser::Analyser;
use scylla_pg_lib::manager::PgManager;

#[tokio::main]
pub async fn main() {
    env_logger::builder().format_timestamp_millis().init();
    let args: Vec<String> = std::env::args().collect();
    let worker_count: i32 = args[1].parse::<i32>().expect("workers cannot be parsed into i32");
    let mut analyser = Analyser::new(10);
    let tx = analyser.tx.clone();
    tokio::spawn(async move {
        analyser.run().await;
    });
    let mut future_list = Vec::new();

    for id in 0..worker_count {
        let worker_id = format!("worker_{id}");
        let tx_clone = tx.clone();
        future_list.push(tokio::spawn(async move {
            start_worker(worker_id, tx_clone).await;
        }));
    }
    while !&future_list.first().unwrap().is_finished() {}
}

pub async fn start_worker(worker_id: String, tx: tokio::sync::mpsc::Sender<u64>) {
    let pgm = PgManager::from_config(&PGConfig::from_env().unwrap()).expect("Error creating PgManager Instance");

    loop {
        let worker_clone = worker_id.clone();
        let instant = tokio::time::Instant::now();
        match pgm.lease_n_tasks("load_test".to_string(), 1, worker_clone, Some(5)).await {
            Err(e) => {
                log::error!("error occurred while leasing tasks {e}");
            }
            Ok(tasks) => {
                for _ in tasks {
                    let _ = tx.send(instant.elapsed().as_millis().try_into().unwrap()).await;
                    // if let Err(e) = pgm.heartbeat_task(t.rn.clone(), t.owner.unwrap(), None, None).await {
                    //     log::error!("error occurred while heartbeat tasks {e}");
                    // }
                    // if let Err(e) = pgm.complete_task(t.rn.clone()).await {
                    //     log::error!("error occurred while complete tasks {e}");
                    // }
                }
            }
        }
    }
}
