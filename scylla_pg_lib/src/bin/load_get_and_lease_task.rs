use scylla_models::{GetTaskModel, TaskStatus};
use scylla_pg_core::config::PGConfig;
use scylla_pg_lib::manager::PgManager;
use std::time::Duration;


#[tokio::main]
pub async fn main() {
    env_logger::builder().format_timestamp_millis().init();
    let args: Vec<String> = std::env::args().collect();
    let worker_count: i32 = args[1].parse::<i32>().expect("workers cannot be parsed into i32");
    let mut future_list = Vec::new();
    for id in 0..worker_count {
        let worker_id = format!("worker_{id}");
        future_list.push(
            tokio::spawn(async move {
            start_worker(worker_id).await;
        })
    );

    }
   while !&future_list.get(0).unwrap().is_finished(){}
    
}



pub async fn start_worker(worker_id: String) {
    let pgm = PgManager::from_config(&PGConfig::from_env().unwrap()).expect("Error creating PgManager Instance");

    loop {
        let worker_clone = worker_id.clone();
        match pgm.fetch_tasks(GetTaskModel{
            limit: Some(200),
            queue: Some("load_test".to_string()),
            status: Some(TaskStatus::Ready),
            worker: None
        }).await {
            Ok(tasks) => {
                if !tasks.is_empty() {
                    let random_index = rand::random_range(0..tasks.len());
                    match pgm.lease_task(tasks.get(random_index).unwrap().rn.clone(), worker_clone, Some(5)).await {
                        Err(e) => {
                            log::error!("error occurred while leasing task {e}");
                        },
                        Ok(t) => {
                            // if let Err(e) = pgm.heartbeat_task(t.rn.clone(), t.owner.unwrap(), None, None).await {
                            //     log::error!("error occurred while heartbeat tasks {e}");
                            // }
                            // tokio::time::sleep(Duration::from_millis(1000)).await;
                            if let Err(e) = pgm.complete_task(t.rn.clone()).await {
                                log::error!("error occurred while complete tasks {e}");
                            }
                        }
                    }
                }

            },
            Err(e) => {
                log::error!("error occurred while fetching tasks {e}"); 
            }
        }
    }
}