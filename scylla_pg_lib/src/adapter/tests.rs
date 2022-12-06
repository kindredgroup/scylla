

// $coverage:ignore-start
use crate::adapter::*;

#[test]
fn prepare_insert_task_cases() {
    let t = Task::default();
    assert_eq!(prepare_insert_task(&t), serde_json::to_value(&t).unwrap());
}

#[test]
fn handle_insert_return_cases() {
    let original_task = Task {
        rn: "123".to_string(),
        ..Task::default()
    };

    assert_eq!(handle_insert_return(vec![], &original_task).is_err(), true);
    assert_eq!(
        handle_insert_return(vec![], &original_task).unwrap_err().to_string(),
        PgAdapterError::DuplicateTask(original_task.rn.to_owned()).to_string()
    );
    let ret_t = Task {
        rn: "123".to_string(),
        status: scylla_models::TaskStatus::Running,
        ..Task::default()
    };
    let ret_t1 = Task {
        rn: "123".to_string(),
        status: scylla_models::TaskStatus::Running,
        ..Task::default()
    };
    // In case single item is returned from db. That will be retruned back
    assert_eq!(handle_insert_return(vec![ret_t.clone()], &original_task).unwrap(), ret_t);
    // In case more than item is returned from db. it's returning first item. Implement transactions
    assert_eq!(handle_insert_return(vec![ret_t.clone(), ret_t1.clone()], &original_task).unwrap(), ret_t);
}