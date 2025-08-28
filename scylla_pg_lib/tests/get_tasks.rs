use scylla_models::TaskStatus;
use scylla_models::{AddTaskModel, GetTaskModel, TaskError};
mod common;

#[tokio::test]
#[ignore]
async fn get_running_tasks() {
    // truncate table before use
    common::truncate_table().await;
    let pgm = common::get_pg_manager().await;
    let atm = AddTaskModel {
        rn: "add_test_1".to_string(),
        queue: "add_test".to_string(),
        priority: 1,
        spec: serde_json::from_str("{\"a\":\"b\"}").unwrap(),
    };
    pgm.insert_task(atm).await.unwrap();

    pgm.lease_task("add_test_1".to_string(), "worker".to_string(), None).await.unwrap();

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
    let pgm = common::get_pg_manager().await;
    let atm = AddTaskModel {
        rn: "add_test_1".to_string(),
        queue: "add_test".to_string(),
        priority: 1,
        spec: serde_json::from_str("{\"a\":\"b\"}").unwrap(),
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
    let pgm = common::get_pg_manager().await;
    let atm = AddTaskModel {
        rn: "add_test_1".to_string(),
        queue: "add_test".to_string(),
        priority: 1,
        spec: serde_json::from_str("{\"a\":\"b\"}").unwrap(),
    };
    pgm.insert_task(atm).await.unwrap();
    pgm.lease_task("add_test_1".to_string(), "worker".to_string(), None).await.unwrap();
    pgm.complete_task("add_test_1".to_string(), None).await.unwrap();

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

#[tokio::test]
#[ignore]
async fn get_ready_tasks() {
    // truncate table before use
    common::truncate_table().await;
    let pgm = common::get_pg_manager().await;
    let atm = AddTaskModel {
        rn: "add_test_1".to_string(),
        queue: "add_test".to_string(),
        priority: 1,
        spec: serde_json::from_str("{\"a\":\"b\"}").unwrap(),
    };
    // get before insert
    let gtm_0 = GetTaskModel {
        status: Some(TaskStatus::Ready),
        worker: None,
        queue: None,
        limit: None,
    };
    assert_eq!(pgm.fetch_tasks(gtm_0).await.unwrap().len(), 0);
    pgm.insert_task(atm).await.unwrap();
    let gtm = GetTaskModel {
        status: Some(TaskStatus::Ready),
        worker: None,
        queue: None,
        limit: None,
    };
    let ready_tasks = pgm.fetch_tasks(gtm).await.unwrap();
    assert_eq!(ready_tasks.len(), 1);
    assert_eq!(ready_tasks[0].status, TaskStatus::Ready);
    assert_eq!(ready_tasks[0].owner, None); // No owner assigned in ready state

    // truncate table before use
    common::truncate_table().await;
}

#[tokio::test]
#[ignore]
async fn get_aborted_tasks() {
    // truncate table before use
    common::truncate_table().await;
    let pgm = common::get_pg_manager().await;
    let atm = AddTaskModel {
        rn: "add_test_1".to_string(),
        queue: "add_test".to_string(),
        priority: 1,
        spec: serde_json::from_str("{\"a\":\"b\"}").unwrap(),
    };
    // get before insert
    let gtm_0 = GetTaskModel {
        status: Some(TaskStatus::Aborted),
        worker: None,
        queue: None,
        limit: None,
    };
    assert_eq!(pgm.fetch_tasks(gtm_0).await.unwrap().len(), 0);
    pgm.insert_task(atm).await.unwrap();
    pgm.lease_task("add_test_1".to_string(), "worker".to_string(), None).await.unwrap();
    pgm.abort_task(
        "add_test_1".to_string(),
        TaskError {
            code: "101".to_string(),
            description: "basic validation failed".to_string(),
            args: serde_json::Value::default(),
        },
    )
    .await
    .unwrap();
    let gtm = GetTaskModel {
        status: Some(TaskStatus::Aborted),
        worker: None,
        queue: None,
        limit: None,
    };
    let aborted_tasks = pgm.fetch_tasks(gtm).await.unwrap();
    assert_eq!(aborted_tasks.len(), 1);
    assert_eq!(aborted_tasks[0].status, TaskStatus::Aborted);
    assert_eq!(aborted_tasks[0].owner, Some("worker".to_string()));
    assert_eq!(aborted_tasks[0].errors[0].code, "101".to_string());

    // truncate table before use
    common::truncate_table().await;
}

#[tokio::test]
#[ignore]
async fn get_worker_tasks() {
    // truncate table before use
    common::truncate_table().await;
    let pgm = common::get_pg_manager().await;
    let atm_1 = AddTaskModel {
        rn: "add_test_1".to_string(),
        queue: "add_test".to_string(),
        priority: 1,
        spec: serde_json::from_str("{\"a\":\"b\"}").unwrap(),
    };
    let atm_2 = AddTaskModel {
        rn: "add_test_2".to_string(),
        queue: "add_test".to_string(),
        priority: 1,
        spec: serde_json::from_str("{\"a\":\"b\"}").unwrap(),
    };
    pgm.insert_task(atm_1).await.unwrap();
    pgm.insert_task(atm_2).await.unwrap();
    // get before leasing
    let gtm_1 = GetTaskModel {
        status: None,
        worker: Some("worker".to_string()),
        queue: None,
        limit: None,
    };
    assert_eq!(pgm.fetch_tasks(gtm_1).await.unwrap().len(), 0);
    pgm.lease_task("add_test_1".to_string(), "worker".to_string(), None).await.unwrap();
    pgm.lease_task("add_test_2".to_string(), "worker".to_string(), None).await.unwrap();
    let gtm_2 = GetTaskModel {
        status: None,
        worker: Some("worker".to_string()),
        queue: None,
        limit: None,
    };
    let fetched_tasks_2 = pgm.fetch_tasks(gtm_2).await.unwrap();
    assert_eq!(fetched_tasks_2.len(), 2);
    assert_eq!(fetched_tasks_2[0].owner, Some("worker".to_string()));
    assert_eq!(fetched_tasks_2[1].owner, Some("worker".to_string()));
    let gtm_3 = GetTaskModel {
        status: None,
        worker: Some("worker".to_string()),
        queue: None,
        limit: Some(1),
    };
    let fetched_tasks_3 = pgm.fetch_tasks(gtm_3).await.unwrap();
    assert_eq!(fetched_tasks_3.len(), 1);
    assert_eq!(fetched_tasks_3[0].owner, Some("worker".to_string()));

    // truncate table before use
    common::truncate_table().await;
}

#[tokio::test]
#[ignore]
async fn get_queue_tasks() {
    // truncate table before use
    common::truncate_table().await;
    let pgm = common::get_pg_manager().await;
    let atm_1 = AddTaskModel {
        rn: "add_test_1".to_string(),
        queue: "add_test".to_string(),
        priority: 1,
        spec: serde_json::from_str("{\"a\":\"b\"}").unwrap(),
    };
    let atm_2 = AddTaskModel {
        rn: "add_test_2".to_string(),
        queue: "add_test".to_string(),
        priority: 1,
        spec: serde_json::from_str("{\"a\":\"b\"}").unwrap(),
    };

    // get before inserting
    let gtm_1 = GetTaskModel {
        status: None,
        worker: None,
        queue: Some("add_test".to_string()),
        limit: None,
    };
    assert_eq!(pgm.fetch_tasks(gtm_1).await.unwrap().len(), 0);
    pgm.insert_task(atm_1).await.unwrap();
    pgm.insert_task(atm_2).await.unwrap();
    let gtm_2 = GetTaskModel {
        status: None,
        worker: None,
        queue: Some("add_test".to_string()),
        limit: None,
    };
    let fetched_tasks_2 = pgm.fetch_tasks(gtm_2).await.unwrap();
    assert_eq!(fetched_tasks_2.len(), 2);
    assert_eq!(fetched_tasks_2[0].queue, "add_test".to_string());
    assert_eq!(fetched_tasks_2[1].queue, "add_test".to_string());
    let gtm_3 = GetTaskModel {
        status: None,
        worker: None,
        queue: Some("add_test".to_string()),
        limit: Some(1),
    };
    let fetched_tasks_3 = pgm.fetch_tasks(gtm_3).await.unwrap();
    assert_eq!(fetched_tasks_3.len(), 1);
    assert_eq!(fetched_tasks_3[0].queue, "add_test".to_string());

    let gtm_4 = GetTaskModel {
        status: None,
        worker: None,
        queue: Some("add_testwww".to_string()),
        limit: None,
    };
    assert_eq!(pgm.fetch_tasks(gtm_4).await.unwrap().len(), 0);

    // truncate table before use
    common::truncate_table().await;
}
