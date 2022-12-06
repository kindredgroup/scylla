mod config;
mod utils;

use scylla_models::{GetTaskModel, TaskStatus};
use scylla_pg_lib::manager::{PgManager, DbConfig};
// use scylla_pg_lib::error::PgAdapterError;
use utils::filter_expired_tasks;
use std::collections::HashMap;
use std::time::Duration;
use crate::config::Config;


//todo: add logger support in here
// retry mechanism

pub async fn monitor_tasks() {
    
    let pgm = PgManager::from_config(get_db_config()).expect("Error creating PgManager Instance");
    loop {
      tokio::time::sleep(Duration::from_secs(5)).await;
      let gtm = GetTaskModel {
        status: Some(TaskStatus::Running),
        worker: None,
        queue: None,
        limit: None,
      };
      match pgm.fetch_tasks(gtm).await {
        Ok(tasks) => {
          let expired_tasks = filter_expired_tasks(tasks);
          let mut res = HashMap::new();
          for task in expired_tasks {
            res.insert(task.rn.clone(), pgm.reset_task(task.rn).await);
          }
          for (rn, task_res) in &res {
            match task_res {
              Ok(_) => println!("task with {} has been reset to ready state ", rn),
              Err(e) => println!("task with {} in reset failed to reset because of error: {:?}", rn, e),
            }
          }
        }
        Err(e) => println!("error e {:?}", e),
      }
    }
}

fn get_db_config() -> DbConfig {
    dotenv::dotenv().ok();

    let config = Config::from_env().unwrap();
    DbConfig {
      port: config.pgport,
      db_name: config.pgdatabase,
      password: config.pgpassword,
      user: config.pguser,
      host: config.pghost,
    }
}