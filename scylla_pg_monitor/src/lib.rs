// $coverage:ignore-start
//! Ignored from coverage because of real database interactions. covered as part of component tests
mod utils;

use scylla_models::{GetTaskModel, Task, TaskStatus};
use scylla_pg_core::config::PGConfig;
use scylla_pg_lib::error::PgAdapterError;
use scylla_pg_lib::manager::PgManager;
use std::collections::HashMap;
use std::time::Duration;
use utils::filter_expired_tasks;

fn handle_insert_return(res: &HashMap<String, Result<Task, PgAdapterError>>) {
    for (rn, task_res) in res {
        match task_res {
            Ok(_) => println!("task with {rn} has been reset to ready state "),
            Err(e) => eprintln!("task with {rn} in reset failed to reset because of error: {e:?}"),
        }
    }
}

fn get_running_query_model() -> GetTaskModel {
    GetTaskModel {
        status: Some(TaskStatus::Running),
        worker: None,
        queue: None,
        limit: None,
    }
}

/// # Panics
/// In case invalid env config is passed.
pub async fn monitor_tasks() {
    let pgm = PgManager::from_config(&PGConfig::from_env().unwrap()).expect("Error creating PgManager Instance");
    loop {
        tokio::time::sleep(Duration::from_secs(5)).await;
        reset_tasks(&pgm).await;
    }
}

async fn reset_tasks(pgm: &PgManager) {
    match pgm.fetch_tasks(get_running_query_model()).await {
        Ok(tasks) => {
            let expired_tasks = filter_expired_tasks(tasks);
            let mut res: HashMap<String, Result<Task, PgAdapterError>> = HashMap::new();
            for task in expired_tasks {
                res.insert(task.rn.clone(), pgm.reset_task(task.rn).await);
            }
            handle_insert_return(&res);
        }
        Err(e) => eprintln!("error e {e:?}"),
    }
}
