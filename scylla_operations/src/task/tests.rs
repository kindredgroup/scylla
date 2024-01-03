use chrono::{Duration, Utc};
// $coverage:ignore-start
use crate::task::{AddTaskModel, ScyllaOperations, ScyllaOperationsError};
use scylla_models::{Task, TaskStatus, UpdateOperation, UpdateTaskModel};

#[test]
fn insert_returns_task() {
    let add_task_model = AddTaskModel {
        rn: "1234".to_string(),
        priority: 1,
        queue: "ss".to_string(),
        spec: serde_json::Value::default(),
    };
    let default_task: Task = Task::default();
    let returned_task = ScyllaOperations::add_task_operation(&add_task_model);
    assert_eq!(&returned_task.rn, &add_task_model.rn);
    assert_eq!(&returned_task.priority, &add_task_model.priority);
    assert_eq!(&returned_task.queue, &add_task_model.queue);
    assert_eq!(&returned_task.spec, &add_task_model.spec);
    // default values assigned
    assert_eq!(&returned_task.progress, &default_task.progress);
    assert_eq!(&returned_task.errors, &default_task.errors);
    assert_eq!(&returned_task.history, &default_task.history);
    assert_eq!(&returned_task.owner, &default_task.owner);
    assert_eq!(&returned_task.status, &default_task.status);
}
#[test]
fn update_task_calls_get_and_update() {
    let utm = UpdateTaskModel {
        rn: "unique_id".to_string(),
        operation: UpdateOperation::Lease,
        status: Some(TaskStatus::Running),
        worker: Some("worker1".to_string()),
        error: None,
        progress: None,
        task_timeout_in_secs: None,
    };
    let task_to_update = Task {
        rn: "unique_id".to_string(),
        status: TaskStatus::Ready,
        ..Task::default()
    };
    let task = ScyllaOperations::update_task_operation(&utm, task_to_update).unwrap();
    assert_eq!(task.rn, "unique_id".to_string());
    assert_eq!(task.status, TaskStatus::Running);
    assert_eq!(task.owner, Some("worker1".to_string()));
    assert!(task.deadline.unwrap() < Utc::now() + Duration::seconds(11))
}

#[test]
fn update_task_returns_scylla_op_error() {
    let task_to_update = Task {
        rn: "unique_id".to_string(),
        status: TaskStatus::Ready,
        ..Task::default()
    };
    let utm = UpdateTaskModel {
        rn: "unique_id".to_string(),
        operation: UpdateOperation::Status,
        status: Some(TaskStatus::Completed),
        worker: None,
        error: None,
        progress: None,
        task_timeout_in_secs: None,
    };

    assert_eq!(
        ScyllaOperations::update_task_operation(&utm, task_to_update),
        Err(ScyllaOperationsError::InvalidStatusTransition(TaskStatus::Ready, vec![TaskStatus::Cancelled]))
    );
}
