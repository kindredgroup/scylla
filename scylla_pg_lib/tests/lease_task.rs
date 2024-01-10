mod common;

use scylla_models::{AddTaskModel, TaskHistoryType, TaskStatus};

#[tokio::test]
#[ignore]
async fn lease_task() {
    // truncate table before use
    common::truncate_table().await;
    let pgm = common::get_pg_manager().await;
    let atm = AddTaskModel {
        rn: "lease_success".to_string(),
        queue: "test".to_string(),
        priority: 1,
        spec: serde_json::from_str("{\"a\":\"b\"}").unwrap(),
    };

    pgm.insert_task(atm).await.unwrap();
    let leased_task = pgm.lease_task("lease_success".to_string(), "test_worker".to_string(), None).await.unwrap();
    assert_eq!(leased_task.status, TaskStatus::Running);
    // truncate table after use
    common::truncate_table().await;
}

#[tokio::test]
#[ignore]
async fn lease_n_tasks() {
    // truncate table before use
    common::truncate_table().await;
    let pgm = common::get_pg_manager().await;
    let atm1 = AddTaskModel {
        rn: "lease_success1".to_string(),
        queue: "test".to_string(),
        priority: 100,
        spec: serde_json::from_str("{\"a\":\"b\"}").unwrap(),
    };
    let atm2 = AddTaskModel {
        rn: "lease_fail2".to_string(),
        queue: "test".to_string(),
        priority: 10,
        spec: serde_json::from_str("{\"a\":\"b\"}").unwrap(),
    };
    let atm3 = AddTaskModel {
        rn: "lease_success3".to_string(),
        queue: "test".to_string(),
        priority: 50,
        spec: serde_json::from_str("{\"a\":\"b\"}").unwrap(),
    };
    let atm4 = AddTaskModel {
        rn: "lease_fail4".to_string(),
        queue: "testing".to_string(), // different queue
        priority: 120,
        spec: serde_json::from_str("{\"a\":\"b\"}").unwrap(),
    };

    pgm.insert_task(atm1).await.unwrap();
    pgm.insert_task(atm2).await.unwrap();
    pgm.insert_task(atm3).await.unwrap();
    pgm.insert_task(atm4).await.unwrap();
    let leased_tasks = pgm.lease_n_tasks("test".to_string(), 2, "worker".to_string(), None).await.unwrap();
    assert_eq!(leased_tasks.len(), 2);
    assert_ne!(leased_tasks.iter().position(|t| t.rn == *"lease_success1"), None);
    assert_ne!(leased_tasks.iter().position(|t| t.rn == *"lease_success3"), None);
    assert_eq!(leased_tasks.iter().position(|t| t.rn == *"lease_fail2"), None);
    assert_eq!(leased_tasks.iter().position(|t| t.rn == *"lease_fail4"), None);
    assert_eq!(leased_tasks[0].owner, Some("worker".to_string()));
    assert_eq!(leased_tasks[0].history[0].typ, TaskHistoryType::Assignment);
    assert_eq!(leased_tasks[0].history[0].worker, "worker".to_string());
    assert_eq!(leased_tasks[0].history.len(), 1);
    assert_eq!(leased_tasks[0].status, TaskStatus::Running);
    assert_eq!(leased_tasks[1].status, TaskStatus::Running);
    // truncate table after use
    common::truncate_table().await;
}
