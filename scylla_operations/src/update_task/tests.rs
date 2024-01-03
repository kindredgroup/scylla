// $coverage:ignore-start
use crate::error::ScyllaOperationsError;
use crate::update_task::*;
use scylla_models::{Task, TaskError, TaskStatus, UpdateOperation, UpdateTaskModel};

#[test]
fn validate_status_failure_scenarios() {
    /*********************************/
    // status is mandatory for status operation
    let utm_missing_status = UpdateTaskModel {
        operation: UpdateOperation::Status,
        status: None,
        rn: "abc".to_string(),
        error: None,
        worker: None,
        progress: None,
    };

    assert_eq!(
        validate_status_operation(&Task::default(), &utm_missing_status),
        Err(ScyllaOperationsError::MandatoryFieldMissing("status".to_string(), UpdateOperation::Status))
    );
    /*********************************/
    // Ready  can only be moved to Cancelled state
    let utm_aborted_status = UpdateTaskModel {
        operation: UpdateOperation::Status,
        status: Some(TaskStatus::Aborted),
        rn: "abc".to_string(),
        error: None,
        worker: None,
        progress: None,
    };
    let utm_running_status = UpdateTaskModel {
        operation: UpdateOperation::Status,
        status: Some(TaskStatus::Running),
        rn: "abc".to_string(),
        error: None,
        worker: None,
        progress: None,
    };
    let utm_completed_status = UpdateTaskModel {
        operation: UpdateOperation::Status,
        status: Some(TaskStatus::Completed),
        rn: "abc".to_string(),
        error: None,
        worker: None,
        progress: None,
    };
    let t_ready = Task {
        status: TaskStatus::Ready,
        ..Task::default()
    };
    assert_eq!(
        validate_status_operation(&t_ready, &utm_aborted_status),
        Err(ScyllaOperationsError::InvalidStatusTransition(TaskStatus::Ready, vec![TaskStatus::Cancelled]))
    );
    assert_eq!(
        validate_status_operation(&t_ready, &utm_completed_status),
        Err(ScyllaOperationsError::InvalidStatusTransition(TaskStatus::Ready, vec![TaskStatus::Cancelled]))
    );
    assert_eq!(
        validate_status_operation(&t_ready, &utm_running_status),
        Err(ScyllaOperationsError::InvalidStatusTransition(TaskStatus::Ready, vec![TaskStatus::Cancelled]))
    );

    /*********************************/
    // Running cannot be moved to Ready State under status change operation
    let utm_ready_status = UpdateTaskModel {
        operation: UpdateOperation::Status,
        status: Some(TaskStatus::Ready),
        rn: "abc".to_string(),
        error: None,
        worker: None,
        progress: None,
    };
    let t_running = Task {
        status: TaskStatus::Running,
        ..Task::default()
    };
    assert_eq!(
        validate_status_operation(&t_running, &utm_ready_status),
        Err(ScyllaOperationsError::InvalidStatusTransition(
            TaskStatus::Running,
            vec![TaskStatus::Completed, TaskStatus::Cancelled, TaskStatus::Aborted]
        ))
    );

    /*********************************/
    // Terminal States cannot be updated
    let utm_ready = UpdateTaskModel {
        operation: UpdateOperation::Status,
        status: Some(TaskStatus::Ready),
        rn: "abc".to_string(),
        error: None,
        worker: None,
        progress: None,
    };
    let t_aborted = Task {
        status: TaskStatus::Aborted,
        ..Task::default()
    };
    let t_cancelled = Task {
        status: TaskStatus::Cancelled,
        ..Task::default()
    };
    let t_completed = Task {
        status: TaskStatus::Completed,
        ..Task::default()
    };
    assert_eq!(
        validate_status_operation(&t_aborted, &utm_ready),
        Err(ScyllaOperationsError::TerminalTaskStatus(
            TaskStatus::Aborted,
            vec![TaskStatus::Ready, TaskStatus::Running]
        ))
    );
    assert_eq!(
        validate_status_operation(&t_cancelled, &utm_ready),
        Err(ScyllaOperationsError::TerminalTaskStatus(
            TaskStatus::Cancelled,
            vec![TaskStatus::Ready, TaskStatus::Running]
        ))
    );
    assert_eq!(
        validate_status_operation(&t_completed, &utm_ready),
        Err(ScyllaOperationsError::TerminalTaskStatus(
            TaskStatus::Completed,
            vec![TaskStatus::Ready, TaskStatus::Running]
        ))
    );

    /*********************************/
    //  Error needs to be passed down in case of aborting a task
    let utm_aborted = UpdateTaskModel {
        operation: UpdateOperation::Status,
        status: Some(TaskStatus::Aborted),
        rn: "abc".to_string(),
        error: None,
        worker: None,
        progress: None,
    };
    let t_running = Task {
        status: TaskStatus::Running,
        ..Task::default()
    };
    assert_eq!(
        validate_status_operation(&t_running, &utm_aborted),
        Err(ScyllaOperationsError::MandatoryFieldMissing("error".to_string(), UpdateOperation::Status))
    );
}

#[test]
fn validate_state_success_scenarios() {
    /*********************************/
    //  Ready --> Cancelled
    let utm_cancelled_status = UpdateTaskModel {
        operation: UpdateOperation::Status,
        status: Some(TaskStatus::Cancelled),
        rn: "abc".to_string(),
        error: None,
        worker: None,
        progress: None,
    };
    let t_ready = Task {
        status: TaskStatus::Ready,
        ..Task::default()
    };

    validate_status_operation(&t_ready, &utm_cancelled_status).unwrap();

    /*********************************/
    //  Running --> Cancelled
    let utm_cancelled_status = UpdateTaskModel {
        operation: UpdateOperation::Status,
        status: Some(TaskStatus::Cancelled),
        rn: "abc".to_string(),
        error: None,
        worker: None,
        progress: None,
    };
    let utm_completed_status = UpdateTaskModel {
        operation: UpdateOperation::Status,
        status: Some(TaskStatus::Cancelled),
        rn: "abc".to_string(),
        error: None,
        worker: None,
        progress: None,
    };
    let utm_aborted_status = UpdateTaskModel {
        operation: UpdateOperation::Status,
        status: Some(TaskStatus::Cancelled),
        rn: "abc".to_string(),
        error: Some(TaskError {
            code: "123".to_string(),
            args: serde_json::from_str("{\"a\": \"b\"}").unwrap(),
            description: "new task superceeded current one.".to_string(),
        }),
        worker: None,
        progress: None,
    };
    let t_running = Task {
        status: TaskStatus::Running,
        owner: Some("worker1".to_string()),
        ..Task::default()
    };

    validate_status_operation(&t_running, &utm_cancelled_status).unwrap();
    validate_status_operation(&t_running, &utm_completed_status).unwrap();
    validate_status_operation(&t_running, &utm_aborted_status).unwrap();
}

#[test]
fn prepare_status_task_cases() {
    /*********************************/
    //  Aborted --> error being extened in list

    let utm_aborted = UpdateTaskModel {
        operation: UpdateOperation::Status,
        status: Some(TaskStatus::Aborted),
        rn: "abc".to_string(),
        error: Some(TaskError {
            code: "123".to_string(),
            args: serde_json::from_str("{\"a\": \"b\"}").unwrap(),
            description: "new task superceeded current one.".to_string(),
        }),
        worker: None,
        progress: None,
    };
    let task = Task {
        errors: vec![TaskError {
            code: "different_error".to_string(),
            args: serde_json::from_str("{\"a\": \"b\"}").unwrap(),
            description: "new task superceeded current one.".to_string(),
        }],
        ..Task::default()
    };
    let prepared_task = prepare_status_task(task, &utm_aborted);
    assert_eq!(prepared_task.status, utm_aborted.status.unwrap());
    // error added to list
    assert_eq!(prepared_task.errors.len(), 2);

    /*********************************/
    //  Any other cased --> error does not being added to list.

    let utm_cancelled = UpdateTaskModel {
        operation: UpdateOperation::Status,
        status: Some(TaskStatus::Cancelled),
        rn: "abc".to_string(),
        error: Some(TaskError {
            code: "123".to_string(),
            args: serde_json::from_str("{\"a\": \"b\"}").unwrap(),
            description: "new task superceeded current one.".to_string(),
        }),
        worker: None,
        progress: None,
    };
    let task = Task {
        errors: vec![TaskError {
            code: "different_error".to_string(),
            args: serde_json::from_str("{\"a\": \"b\"}").unwrap(),
            description: "new task superceeded current one.".to_string(),
        }],
        ..Task::default()
    };
    let prepared_task = prepare_status_task(task, &utm_cancelled);
    assert_eq!(prepared_task.status, utm_cancelled.status.unwrap());
    // error added to list
    assert_eq!(prepared_task.errors.len(), 1);
}

#[test]
fn validate_yield_operation_cases() {
    /*********************************/
    //  Only Running task can be yielded

    let t_ready = Task {
        status: TaskStatus::Ready,
        ..Task::default()
    };
    let t_running = Task {
        status: TaskStatus::Running,
        ..Task::default()
    };
    let t_aborted = Task {
        status: TaskStatus::Aborted,
        ..Task::default()
    };
    let t_cancelled = Task {
        status: TaskStatus::Cancelled,
        ..Task::default()
    };
    let t_completed = Task {
        status: TaskStatus::Completed,
        ..Task::default()
    };
    assert_eq!(
        validate_yield_operation(&t_ready),
        Err(ScyllaOperationsError::InvalidOperation(
            UpdateOperation::Yield,
            TaskStatus::Running,
            TaskStatus::Ready,
        ))
    );
    assert_eq!(
        validate_yield_operation(&t_aborted),
        Err(ScyllaOperationsError::InvalidOperation(
            UpdateOperation::Yield,
            TaskStatus::Running,
            TaskStatus::Aborted,
        ))
    );
    assert_eq!(
        validate_yield_operation(&t_cancelled),
        Err(ScyllaOperationsError::InvalidOperation(
            UpdateOperation::Yield,
            TaskStatus::Running,
            TaskStatus::Cancelled,
        ))
    );
    assert_eq!(
        validate_yield_operation(&t_completed),
        Err(ScyllaOperationsError::InvalidOperation(
            UpdateOperation::Yield,
            TaskStatus::Running,
            TaskStatus::Completed,
        ))
    );
    // only running task can be yielded
    validate_yield_operation(&t_running).unwrap();
}

#[test]
fn prepare_yield_task_cases() {
    let task = Task {
        status: TaskStatus::Running,
        owner: Some("worker1".to_string()),
        progress: 0.4,
        ..Task::default()
    };
    let prepared_task = prepare_yield_task(task);
    assert_eq!(prepared_task.history.len(), 1);
    assert_eq!(prepared_task.history[0].typ, TaskHistoryType::Yield);
    assert_eq!(prepared_task.history[0].worker, "worker1".to_string());
    assert_eq!(prepared_task.history[0].progress, Some(0.4));
    assert!(prepared_task.deadline.unwrap() < Utc::now());
}

#[test]
fn validate_heart_beat_operation_cases() {
    /*********************************/
    //  Only Running task can be send heart beat

    let t_ready = Task {
        status: TaskStatus::Ready,
        ..Task::default()
    };
    let t_running = Task {
        status: TaskStatus::Running,
        ..Task::default()
    };
    let t_aborted = Task {
        status: TaskStatus::Aborted,
        ..Task::default()
    };
    let t_cancelled = Task {
        status: TaskStatus::Cancelled,
        ..Task::default()
    };
    let t_completed = Task {
        status: TaskStatus::Completed,
        ..Task::default()
    };
    assert_eq!(
        validate_heart_beat_operation(&t_ready),
        Err(ScyllaOperationsError::InvalidOperation(
            UpdateOperation::HeartBeat,
            TaskStatus::Running,
            TaskStatus::Ready,
        ))
    );
    assert_eq!(
        validate_heart_beat_operation(&t_aborted),
        Err(ScyllaOperationsError::InvalidOperation(
            UpdateOperation::HeartBeat,
            TaskStatus::Running,
            TaskStatus::Aborted,
        ))
    );
    assert_eq!(
        validate_heart_beat_operation(&t_cancelled),
        Err(ScyllaOperationsError::InvalidOperation(
            UpdateOperation::HeartBeat,
            TaskStatus::Running,
            TaskStatus::Cancelled,
        ))
    );
    assert_eq!(
        validate_heart_beat_operation(&t_completed),
        Err(ScyllaOperationsError::InvalidOperation(
            UpdateOperation::HeartBeat,
            TaskStatus::Running,
            TaskStatus::Completed,
        ))
    );
    // only running task can send heartbeat
    validate_heart_beat_operation(&t_running).unwrap();
}

#[test]
fn prepare_heart_beat_task_cases() {
    let task_1 = Task::default();
    let utm_without_progress = UpdateTaskModel {
        operation: UpdateOperation::HeartBeat,
        rn: "123".to_string(),
        status: None,
        error: None,
        progress: None,
        worker: None,
    };
    let task_2 = Task::default();
    let utm_with_progress = UpdateTaskModel {
        operation: UpdateOperation::HeartBeat,
        rn: "123".to_string(),
        status: None,
        error: None,
        progress: Some(0.5),
        worker: None,
    };
    let prepared_task = prepare_heart_beat_task(task_1, &utm_without_progress, Duration::seconds(10));
    // just updated
    assert!(Utc::now() - prepared_task.updated < Duration::milliseconds(1));
    assert!(prepared_task.deadline.unwrap() - Duration::seconds(10) - Utc::now() < Duration::milliseconds(1));
    assert_eq!(prepared_task.progress, 0.0);
    let prepared_task = prepare_heart_beat_task(task_2, &utm_with_progress, Duration::seconds(10));
    // just updated
    assert!(Utc::now() - prepared_task.updated < Duration::milliseconds(1));
    assert!(prepared_task.deadline.unwrap() - Duration::seconds(10) - Utc::now() < Duration::milliseconds(1));
    assert_eq!(prepared_task.progress, 0.5);
}

#[test]
fn validate_lease_operation_cases() {
    /*********************************/
    //  Only Ready task can be leased

    let t_ready = Task {
        status: TaskStatus::Ready,
        ..Task::default()
    };
    let t_running = Task {
        status: TaskStatus::Running,
        ..Task::default()
    };
    let t_aborted = Task {
        status: TaskStatus::Aborted,
        ..Task::default()
    };
    let t_cancelled = Task {
        status: TaskStatus::Cancelled,
        ..Task::default()
    };
    let t_completed = Task {
        status: TaskStatus::Completed,
        ..Task::default()
    };
    let utm_with_worker = UpdateTaskModel {
        operation: UpdateOperation::Lease,
        rn: "123".to_string(),
        status: None,
        error: None,
        progress: None,
        worker: Some("worker".to_string()),
    };
    let utm_without_worker = UpdateTaskModel {
        operation: UpdateOperation::Lease,
        rn: "123".to_string(),
        status: None,
        error: None,
        progress: None,
        worker: None,
    };
    assert_eq!(
        validate_lease_operation(&t_running, &utm_without_worker),
        Err(ScyllaOperationsError::InvalidOperation(
            UpdateOperation::Lease,
            TaskStatus::Ready,
            TaskStatus::Running,
        ))
    );
    assert_eq!(
        validate_lease_operation(&t_completed, &utm_without_worker),
        Err(ScyllaOperationsError::InvalidOperation(
            UpdateOperation::Lease,
            TaskStatus::Ready,
            TaskStatus::Completed,
        ))
    );
    assert_eq!(
        validate_lease_operation(&t_cancelled, &utm_without_worker),
        Err(ScyllaOperationsError::InvalidOperation(
            UpdateOperation::Lease,
            TaskStatus::Ready,
            TaskStatus::Cancelled,
        ))
    );
    assert_eq!(
        validate_lease_operation(&t_aborted, &utm_without_worker),
        Err(ScyllaOperationsError::InvalidOperation(
            UpdateOperation::Lease,
            TaskStatus::Ready,
            TaskStatus::Aborted,
        ))
    );
    assert_eq!(
        validate_lease_operation(&t_ready, &utm_without_worker),
        Err(ScyllaOperationsError::MandatoryFieldMissing("worker".to_string(), UpdateOperation::Lease))
    );
    // only ready task with worker can be leased out
    validate_lease_operation(&t_ready, &utm_with_worker).unwrap();
}

#[test]

fn prepare_lease_task_cases() {
    let utm = UpdateTaskModel {
        operation: UpdateOperation::Lease,
        rn: "123".to_string(),
        status: None,
        error: None,
        progress: None,
        worker: Some("worker".to_string()),
    };
    let t = Task {
        status: TaskStatus::Ready,
        history: vec![TaskHistory {
            typ: TaskHistoryType::Timeout,
            progress: Some(0.7),
            time: Utc::now() - Duration::minutes(1),
            worker: "worker1".to_string(),
        }],
        ..Task::default()
    };
    let prepared_task = prepare_lease_task(t, &utm, Duration::seconds(10));
    assert!(Utc::now() - prepared_task.updated < Duration::milliseconds(1));
    assert_eq!(prepared_task.status, TaskStatus::Running);
    assert_eq!(prepared_task.owner.unwrap(), "worker".to_string());
    assert!(prepared_task.deadline.unwrap() - Utc::now() - Duration::seconds(10) < Duration::milliseconds(1));
    assert_eq!(prepared_task.history.len(), 2); // history gets added and not replaced.
    assert_eq!(prepared_task.history[1].typ, TaskHistoryType::Assignment);
    assert!(Utc::now() - prepared_task.history[1].time < Duration::milliseconds(1));
    assert_eq!(prepared_task.history[1].worker, "worker".to_string());
    assert_eq!(prepared_task.history[1].progress, Some(0.0)); // initial progess set as 0.0
}

#[test]
fn validate_reset_operation_cases() {
    /*********************************/
    //  Only Running task can be set to reset

    let t_ready = Task {
        status: TaskStatus::Ready,
        ..Task::default()
    };
    let t_aborted = Task {
        status: TaskStatus::Aborted,
        ..Task::default()
    };
    let t_cancelled = Task {
        status: TaskStatus::Cancelled,
        ..Task::default()
    };
    let t_completed = Task {
        status: TaskStatus::Completed,
        ..Task::default()
    };
    assert_eq!(
        validate_reset_operation(&t_ready),
        Err(ScyllaOperationsError::InvalidOperation(
            UpdateOperation::Reset,
            TaskStatus::Running,
            TaskStatus::Ready,
        ))
    );
    assert_eq!(
        validate_reset_operation(&t_cancelled),
        Err(ScyllaOperationsError::InvalidOperation(
            UpdateOperation::Reset,
            TaskStatus::Running,
            TaskStatus::Cancelled,
        ))
    );
    assert_eq!(
        validate_reset_operation(&t_completed),
        Err(ScyllaOperationsError::InvalidOperation(
            UpdateOperation::Reset,
            TaskStatus::Running,
            TaskStatus::Completed,
        ))
    );
    assert_eq!(
        validate_reset_operation(&t_aborted),
        Err(ScyllaOperationsError::InvalidOperation(
            UpdateOperation::Reset,
            TaskStatus::Running,
            TaskStatus::Aborted,
        ))
    );
    /*********************************/
    // Running status
    let t_running_no_deadline = Task {
        status: TaskStatus::Running,
        ..Task::default()
    };
    let t_running_future_deadline = Task {
        status: TaskStatus::Running,
        deadline: Some(Utc::now() + Duration::seconds(10)),
        ..Task::default()
    };
    let t_running_past_deadline = Task {
        status: TaskStatus::Running,
        deadline: Some(Utc::now() - Duration::seconds(1)),
        ..Task::default()
    };
    //  Running with no deadline

    assert_eq!(
        validate_reset_operation(&t_running_no_deadline),
        Err(ScyllaOperationsError::MandatoryFieldMissing("deadline".to_string(), UpdateOperation::Reset))
    );

    //  Running with valid deadline with still some time left

    assert_eq!(
        validate_reset_operation(&t_running_future_deadline),
        Err(ScyllaOperationsError::ValidationFailed(
            "deadline not yet passed for reset operation".to_string()
        ))
    );

    // only running task with deadline in past can be reset
    validate_reset_operation(&t_running_past_deadline).unwrap();
}

#[test]
fn prepare_reset_task_cases() {
    let t = Task {
        status: TaskStatus::Running,
        deadline: Some(Utc::now() - Duration::seconds(2)),
        progress: 0.8,
        owner: Some("worker2".to_string()),
        updated: Utc::now() - Duration::seconds(2),
        history: vec![TaskHistory {
            typ: TaskHistoryType::Timeout,
            progress: Some(0.7),
            time: Utc::now() - Duration::minutes(1),
            worker: "worker1".to_string(),
        }],
        ..Task::default()
    };
    let t_yielded = Task {
        status: TaskStatus::Running,
        deadline: Some(Utc::now() - Duration::seconds(2)),
        progress: 0.8,
        owner: Some("worker2".to_string()),
        updated: Utc::now() - Duration::seconds(2),
        history: vec![TaskHistory {
            typ: TaskHistoryType::Yield,
            progress: Some(0.7),
            time: Utc::now() - Duration::seconds(2),
            worker: "worker1".to_string(),
        }],
        ..Task::default()
    };
    let pt = prepare_reset_task(t);
    assert_eq!(pt.deadline, None);
    assert_eq!(pt.owner, None);
    assert_eq!(pt.progress, 0.0);
    assert_eq!(pt.status, TaskStatus::Ready);
    assert!(Utc::now() - pt.updated < Duration::milliseconds(1));
    assert_eq!(pt.history.len(), 2);
    assert_eq!(pt.history[1].typ, TaskHistoryType::Timeout);
    assert_eq!(pt.history[1].progress, Some(0.8));
    assert!(Utc::now() - pt.history[1].time < Duration::milliseconds(1));
    assert_eq!(pt.history[1].worker, "worker2".to_string());

    let pt = prepare_reset_task(t_yielded);
    assert_eq!(pt.deadline, None);
    assert_eq!(pt.owner, None);
    assert_eq!(pt.progress, 0.0);
    assert_eq!(pt.status, TaskStatus::Ready);
    assert!(Utc::now() - pt.updated < Duration::milliseconds(1));
    assert_eq!(pt.history.len(), 1);
    assert_eq!(pt.history[0].typ, TaskHistoryType::Yield);
}

#[test]
fn request_handler_cases() {
    /*********************************/
    // Lease operation
    let t = Task {
        status: TaskStatus::Ready,
        ..Task::default()
    };
    let utm = UpdateTaskModel {
        operation: UpdateOperation::Lease,
        rn: "123".to_string(),
        worker: Some("worker1".to_string()),
        error: None,
        progress: None,
        status: None,
    };
    let updated_task = request_handler(t, &utm, Duration::seconds(10)).unwrap();
    assert_eq!(updated_task.status, TaskStatus::Running);
    assert_eq!(updated_task.owner, Some("worker1".to_string()));
    /*********************************/
    // Reset operation
    let t = Task {
        status: TaskStatus::Running,
        deadline: Some(Utc::now() - Duration::seconds(1)),
        owner: Some("worker1".to_string()),
        ..Task::default()
    };
    let utm = UpdateTaskModel {
        operation: UpdateOperation::Reset,
        rn: "123".to_string(),
        worker: None,
        error: None,
        progress: None,
        status: None,
    };
    let updated_task = request_handler(t, &utm, Duration::seconds(10)).unwrap();
    assert_eq!(updated_task.status, TaskStatus::Ready);
    assert_eq!(updated_task.owner, None);
    assert_eq!(updated_task.deadline, None);
    /*********************************/
    // Yield operation
    let t = Task {
        status: TaskStatus::Running,
        deadline: Some(Utc::now() - Duration::seconds(1)),
        owner: Some("worker1".to_string()),
        ..Task::default()
    };
    let utm = UpdateTaskModel {
        operation: UpdateOperation::Yield,
        rn: "123".to_string(),
        worker: None,
        error: None,
        progress: None,
        status: None,
    };
    let updated_task = request_handler(t, &utm, Duration::seconds(10)).unwrap();
    assert_eq!(updated_task.status, TaskStatus::Running);
    assert_eq!(updated_task.owner, Some("worker1".to_string()));
    assert!(Utc::now() - updated_task.deadline.unwrap() > Duration::milliseconds(1));
    /*********************************/
    // Heartbeat operation
    let t = Task {
        status: TaskStatus::Running,
        deadline: Some(Utc::now() - Duration::seconds(1)),
        owner: Some("worker1".to_string()),
        ..Task::default()
    };
    let utm = UpdateTaskModel {
        operation: UpdateOperation::HeartBeat,
        rn: "123".to_string(),
        worker: None,
        error: None,
        progress: Some(0.7),
        status: None,
    };
    let updated_task = request_handler(t, &utm, Duration::seconds(10)).unwrap();
    assert_eq!(updated_task.status, TaskStatus::Running);
    assert_eq!(updated_task.owner, Some("worker1".to_string()));
    assert_eq!(updated_task.progress, 0.7);
    assert!(Utc::now() + Duration::seconds(10) - updated_task.deadline.unwrap() < Duration::milliseconds(1));
    /*********************************/
    // Status operation
    let t = Task {
        status: TaskStatus::Ready,
        ..Task::default()
    };
    let utm = UpdateTaskModel {
        operation: UpdateOperation::Status,
        rn: "123".to_string(),
        worker: None,
        error: None,
        progress: None,
        status: Some(TaskStatus::Cancelled),
    };
    let updated_task = request_handler(t, &utm, Duration::seconds(10)).unwrap();
    assert_eq!(updated_task.status, TaskStatus::Cancelled);
}
