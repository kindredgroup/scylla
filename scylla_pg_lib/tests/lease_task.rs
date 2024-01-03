mod common;
use scylla_models::{AddTaskModel, TaskStatus, UpdateOperation, UpdateTaskModel};
use scylla_pg_lib::error::PgAdapterError;

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
    pgm.lease_task("lease_success".to_string(), "test_worker".to_string(), None).await.unwrap();

    // truncate table after use
    common::truncate_table().await;
}
