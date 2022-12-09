use scylla_models::{GetTaskModel, AddTaskModel};
use scylla_models::TaskStatus;
use scylla_pg_lib::manager::PgManager;
use scylla_pg_core::config::PGConfig;
mod common;

#[tokio::test]
#[ignore]
async fn get_running_tasks() {
        // truncate table before use
        common::truncate_table().await;
    let config = PGConfig::from_env().unwrap();
    let pgm = PgManager::from_config(config).expect("Error creating PgManager Instance");
    let atm = AddTaskModel {
        rn: "add_test_1".to_string(),
        queue: "add_test".to_string(),
        priority: 1,
        spec: serde_json::from_str("{\"a\":\"b\"}").unwrap()

    };   
    pgm.insert_task(atm).await.unwrap();

    pgm.lease_task("add_test_1".to_string(), "worker".to_string() ).await.unwrap();
    
    let gtm = GetTaskModel {
        status: Some(TaskStatus::Running),
        worker: None,
        queue: None,
        limit: None,
      };
    let running_tasks = pgm.fetch_tasks(gtm).await.unwrap();
    assert_eq!(running_tasks.len(), 1);
    assert_eq!(running_tasks[0].status, TaskStatus::Running);
    assert_eq!(running_tasks[0].owner, Some("worker".to_string()));

        // truncate table before use
        common::truncate_table().await;
}

#[tokio::test]
#[ignore]
async fn get_cancelled_tasks() {
        // truncate table before use
        common::truncate_table().await;
    let config = PGConfig::from_env().unwrap();
    let pgm = PgManager::from_config(config).expect("Error creating PgManager Instance");
    let atm = AddTaskModel {
        rn: "add_test_1".to_string(),
        queue: "add_test".to_string(),
        priority: 1,
        spec: serde_json::from_str("{\"a\":\"b\"}").unwrap()

    };   
    pgm.insert_task(atm).await.unwrap();
    pgm.cancel_task("add_test_1".to_string()).await.unwrap();
    
    let gtm = GetTaskModel {
        status: Some(TaskStatus::Cancelled),
        worker: None,
        queue: None,
        limit: None,
      };
    let cancelled_tasks = pgm.fetch_tasks(gtm).await.unwrap();
    assert_eq!(cancelled_tasks.len(), 1);
    assert_eq!(cancelled_tasks[0].status, TaskStatus::Cancelled);

        // truncate table before use
        common::truncate_table().await;
}

#[tokio::test]
#[ignore]
async fn get_completed_tasks() {
        // truncate table before use
        common::truncate_table().await;
    let config = PGConfig::from_env().unwrap();
    let pgm = PgManager::from_config(config).expect("Error creating PgManager Instance");
    let atm = AddTaskModel {
        rn: "add_test_1".to_string(),
        queue: "add_test".to_string(),
        priority: 1,
        spec: serde_json::from_str("{\"a\":\"b\"}").unwrap()

    };   
    pgm.insert_task(atm).await.unwrap();
    pgm.lease_task("add_test_1".to_string(), "worker".to_string()).await.unwrap();
    pgm.complete_task("add_test_1".to_string()).await.unwrap();
    
    let gtm = GetTaskModel {
        status: Some(TaskStatus::Completed),
        worker: None,
        queue: None,
        limit: None,
      };
    let completed_tasks = pgm.fetch_tasks(gtm).await.unwrap();
    assert_eq!(completed_tasks.len(), 1);
    assert_eq!(completed_tasks[0].status, TaskStatus::Completed);
    assert_eq!(completed_tasks[0].owner, Some("worker".to_string())); // owner is kept same and status is updated

        // truncate table before use
        common::truncate_table().await;
}