use scylla_models::AddTaskModel;
use scylla_models::TaskStatus;
use scylla_pg_lib::manager::PgManager;
use scylla_pg_lib::manager::DbConfig;
use scylla_pg_lib::error::PgAdapterError;
#[tokio::test]
#[ignore]
async fn insert_task() {
    let pgm = PgManager::from_config(DbConfig { 
        host: "127.0.0.1".to_string(),
        port: 5432,
        user: "admin".to_string(),
        password: "admin".to_string(),
        db_name: "scylla".to_string() 
    }).expect("Error creating PgManager Instance");
    let atm = AddTaskModel {
        rn: "add_test_1".to_string(),
        queue: "add_test".to_string(),
        priority: 1,
        spec: serde_json::from_str("{\"a\":\"b\"}").unwrap()

    };
    let inserted_task = pgm.insert_task(atm).await.unwrap();
    assert_eq!(inserted_task.rn, "add_test_1".to_string());
    assert_eq!(inserted_task.status, TaskStatus::Ready);
    assert_eq!(inserted_task.queue, "add_test".to_string());
    assert_eq!(inserted_task.priority, 1);
    assert_eq!(inserted_task.spec.to_string(), "{\"a\":\"b\"}".to_string());

    let atm_with_same_rn = AddTaskModel {
        rn: "add_test_1".to_string(),
        queue: "add_test".to_string(),
        priority: 1,
        spec: serde_json::from_str("{\"a\":\"b\"}").unwrap()

    };
    let inserted_task_result = pgm.insert_task(atm_with_same_rn).await;
    assert_eq!(inserted_task_result.is_err(), true);
    assert_eq!(inserted_task_result.err().unwrap().to_string(), PgAdapterError::DuplicateTask("add_test_1".to_string()).to_string());

}