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
fn add_task_operations() {
    let add_task_models = vec![
        AddTaskModel {
            rn: "123".to_string(),
            priority: 1,
            queue: "aa".to_string(),
            spec: serde_json::Value::default(),
        },
        AddTaskModel {
            rn: "456".to_string(),
            priority: 2,
            queue: "bb".to_string(),
            spec: serde_json::Value::default(),
        },
    ];
    let default_task: Task = Task::default();
    let returned_tasks = ScyllaOperations::add_task_operations(&add_task_models);
    // first task
    assert_eq!(&returned_tasks[0].rn, &add_task_models[0].rn);
    assert_eq!(&returned_tasks[0].priority, &add_task_models[0].priority);
    assert_eq!(&returned_tasks[0].queue, &add_task_models[0].queue);
    assert_eq!(&returned_tasks[0].spec, &add_task_models[0].spec);
    assert_eq!(&returned_tasks[0].progress, &default_task.progress);
    assert_eq!(&returned_tasks[0].errors, &default_task.errors);
    assert_eq!(&returned_tasks[0].history, &default_task.history);
    assert_eq!(&returned_tasks[0].owner, &default_task.owner);
    assert_eq!(&returned_tasks[0].status, &default_task.status);
    // second task
    assert_eq!(&returned_tasks[1].rn, &add_task_models[1].rn);
    assert_eq!(&returned_tasks[1].priority, &add_task_models[1].priority);
    assert_eq!(&returned_tasks[1].queue, &add_task_models[1].queue);
    assert_eq!(&returned_tasks[1].spec, &add_task_models[1].spec);
    assert_eq!(&returned_tasks[1].progress, &default_task.progress);
    assert_eq!(&returned_tasks[1].errors, &default_task.errors);
    assert_eq!(&returned_tasks[1].history, &default_task.history);
    assert_eq!(&returned_tasks[1].owner, &default_task.owner);
}

#[test]
fn add_task_operations_dedupes_and_sorts_by_rn() {
    let add_task_models = vec![
        AddTaskModel {
            rn: "456".to_string(),
            priority: 2,
            queue: "bb".to_string(),
            spec: serde_json::Value::default(),
        },
        AddTaskModel {
            rn: "123".to_string(),
            priority: 9,
            queue: "duplicate".to_string(),
            spec: serde_json::json!({"dup": true}),
        },
        AddTaskModel {
            rn: "001".to_string(),
            priority: 3,
            queue: "ab".to_string(),
            spec: serde_json::Value::default(),
        },
        AddTaskModel {
            rn: "123".to_string(),
            priority: 1,
            queue: "aa".to_string(),
            spec: serde_json::Value::default(),
        },
    ];
    let returned_tasks = ScyllaOperations::add_task_operations(&add_task_models);
    assert_eq!(returned_tasks.len(), 3);
    assert_eq!(returned_tasks[0].rn, "001");
    assert_eq!(returned_tasks[0].priority, 3);
    assert_eq!(returned_tasks[0].queue, "ab");
    assert_eq!(returned_tasks[1].rn, "123");
    assert_eq!(returned_tasks[1].priority, 9);
    assert_eq!(returned_tasks[1].queue, "duplicate");
    assert_eq!(returned_tasks[2].rn, "456");
    assert_eq!(returned_tasks[2].priority, 2);
    assert_eq!(returned_tasks[2].queue, "bb");
}

#[test]
fn update_task_calls_get_and_update() {
    let utm = UpdateTaskModel {
        rn: "unique_id".to_string(),
        operation: UpdateOperation::Lease,
        status: Some(TaskStatus::Running),
        worker: Some("worker1".to_string()),
        ..UpdateTaskModel::default()
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
        ..UpdateTaskModel::default()
    };

    assert_eq!(
        ScyllaOperations::update_task_operation(&utm, task_to_update),
        Err(ScyllaOperationsError::InvalidStatusTransition(TaskStatus::Ready, vec![TaskStatus::Cancelled]))
    );
}
