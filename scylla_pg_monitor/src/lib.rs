
mod utils;

use scylla_models::{GetTaskModel, TaskStatus};
use scylla_pg_lib::manager::{PgManager};
use scylla_pg_core::config::PGConfig;
use utils::filter_expired_tasks;
use std::collections::HashMap;
use std::time::Duration;




//todo: add logger support in here
// retry mechanism

pub async fn monitor_tasks() {

    let pgm = PgManager::from_config(PGConfig::from_env().unwrap()).expect("Error creating PgManager Instance");
    loop {
      println!("starting monitor tasks");
      tokio::time::sleep(Duration::from_secs(5)).await;
      let gtm = GetTaskModel {
        status: Some(TaskStatus::Running),
        worker: None,
        queue: None,
        limit: None,
      };
      match pgm.fetch_tasks(gtm).await {
        Ok(tasks) => {
          println!("tasks:: {:?}", tasks);
          let expired_tasks = filter_expired_tasks(tasks);
          println!("expired_tasks:: {:?}", expired_tasks);
          let mut res = HashMap::new();
          for task in expired_tasks {
            res.insert(task.rn.clone(), pgm.reset_task(task.rn).await);
          }
          for (rn, task_res) in &res {
            match task_res {
              Ok(_) => println!("task with {} has been reset to ready state ", rn),
              Err(e) => eprintln!("task with {} in reset failed to reset because of error: {:?}", rn, e),
            }
          }
        }
        Err(e) => println!("error e {:?}", e),
      }
    }
}