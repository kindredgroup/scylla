mod common;

use scylla_models::{AddTaskModel, TaskHistoryType, TaskStatus};

#[tokio::test]
#[ignore]
async fn reset_batch_tasks() {
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
    let _ = pgm.lease_n_tasks("test".to_string(), 2, "worker".to_string(), Some(-1)).await.unwrap();
    let reset_tasks = pgm.reset_batch().await.unwrap();
    assert_eq!(reset_tasks.len(), 2);
    assert_ne!(reset_tasks.iter().position(|t| t.rn == *"lease_success1"), None);
    assert_ne!(reset_tasks.iter().position(|t| t.rn == *"lease_success3"), None);
    assert_eq!(reset_tasks.iter().position(|t| t.rn == *"lease_fail2"), None);
    assert_eq!(reset_tasks.iter().position(|t| t.rn == *"lease_fail4"), None);
    assert_eq!(reset_tasks[0].deadline, None);
    assert_eq!(reset_tasks[1].deadline, None);
    assert_eq!(reset_tasks[0].owner, None);
    assert_eq!(reset_tasks[1].owner, None);
    assert_eq!(reset_tasks[0].progress, 0.0);
    assert_eq!(reset_tasks[1].progress, 0.0);
    assert_eq!(reset_tasks[0].status, TaskStatus::Ready);
    assert_eq!(reset_tasks[1].status, TaskStatus::Ready);
    assert_eq!(reset_tasks[0].history.iter().position(|t| t.typ == TaskHistoryType::Timeout), Some(1));
    assert_eq!(reset_tasks[0].history.iter().position(|t| t.typ == TaskHistoryType::Assignment), Some(0));
    assert_eq!(reset_tasks[1].history.iter().position(|t| t.typ == TaskHistoryType::Timeout), Some(1));
    assert_eq!(reset_tasks[1].history.iter().position(|t| t.typ == TaskHistoryType::Assignment), Some(0));

    // truncate table after use
    common::truncate_table().await;
}
