use crate::error::ScyllaOperationsError;
use chrono::{Duration, Utc};
use scylla_models::{Task, TaskHistory, TaskHistoryType, TaskStatus, TaskStatusExt, UpdateOperation, UpdateTaskModel};

const NON_TERMINAL_STATUSES: &[TaskStatus] = &[TaskStatus::Ready, TaskStatus::Running];

/// # Errors
/// Returns `ScyllaOperationsError`
pub fn validate_status_operation(task: &Task, update_task_model: &UpdateTaskModel) -> Result<(), ScyllaOperationsError> {
    let status_value = update_task_model
        .status
        .as_ref()
        .ok_or_else(|| ScyllaOperationsError::MandatoryFieldMissing("status".to_string(), UpdateOperation::Status))?;

    if !NON_TERMINAL_STATUSES.contains(&task.status) {
        return Err(ScyllaOperationsError::TerminalTaskStatus(task.status.clone(), NON_TERMINAL_STATUSES.into()));
    }

    let allowed_transitions = task.status.allowed_transitions();
    if !allowed_transitions.contains(status_value) {
        return Err(ScyllaOperationsError::InvalidStatusTransition(task.status.clone(), allowed_transitions.into()));
    }

    if *status_value == TaskStatus::Aborted && update_task_model.error.is_none() {
        // errors should be there in case status = aborted
        return Err(ScyllaOperationsError::MandatoryFieldMissing("error".to_string(), UpdateOperation::Status));
    }

    Ok(())
}

/// # Panics
/// In case status is missing. However this function is to be used after `validate_status_task`
fn prepare_status_task(mut task: Task, update_task_model: &UpdateTaskModel) -> Task {
    task.status = update_task_model.status.clone().unwrap();
    task.updated = Utc::now();
    if let Some(error) = update_task_model.error.clone() {
        if task.status == TaskStatus::Aborted {
            task.errors.push(error);
        }
    }
    task
}

/// # Errors
/// Returns `ScyllaOperationsError`
fn validate_yield_operation(task: &Task) -> Result<(), ScyllaOperationsError> {
    if task.status == TaskStatus::Running {
        Ok(())
    } else {
        Err(ScyllaOperationsError::InvalidOperation(
            UpdateOperation::Yield,
            TaskStatus::Running,
            task.status.clone(),
        ))
    }
}

fn prepare_yield_task(mut task: Task) -> Task {
    let task_yield_history = TaskHistory {
        typ: TaskHistoryType::Yield,
        time: Utc::now(),
        worker: task.owner.clone().unwrap(),
        progress: Some(task.progress),
    };
    task.updated = Utc::now();
    task.deadline = Some(Utc::now() - Duration::seconds(1));
    task.history.push(task_yield_history);
    task
}

/// # Errors
/// Returns `ScyllaOperationsError`
fn validate_heart_beat_operation(task: &Task, utm: &UpdateTaskModel) -> Result<(), ScyllaOperationsError> {
    if task.status == TaskStatus::Running && task.owner == utm.worker {
        Ok(())
    } else if task.status != TaskStatus::Running {
        Err(ScyllaOperationsError::InvalidOperation(
            UpdateOperation::HeartBeat,
            TaskStatus::Running,
            task.status.clone(),
        ))
    } else {
        Err(ScyllaOperationsError::ValidationFailed("Only owner can extend the heartbeat.".to_string()))
    }
}

fn prepare_heart_beat_task(mut task: Task, update_task_model: &UpdateTaskModel) -> Task {
    task.updated = Utc::now();
    task.deadline = Some(Utc::now() + Duration::seconds(update_task_model.task_timeout_in_secs.unwrap_or(10)));
    if let Some(progress) = update_task_model.progress {
        task.progress = progress;
    }
    task
}

/// # Errors
/// Returns `ScyllaOperationsError`
fn validate_lease_operation(task: &Task, update_task_model: &UpdateTaskModel) -> Result<(), ScyllaOperationsError> {
    if task.status != TaskStatus::Ready {
        Err(ScyllaOperationsError::InvalidOperation(
            UpdateOperation::Lease,
            TaskStatus::Ready,
            task.status.clone(),
        ))
    } else if update_task_model.worker.is_none() {
        Err(ScyllaOperationsError::MandatoryFieldMissing("worker".to_string(), UpdateOperation::Lease))
    } else {
        Ok(())
    }
}

fn prepare_lease_task(mut task: Task, update_task_model: &UpdateTaskModel) -> Task {
    let task_assignment_history = TaskHistory {
        typ: TaskHistoryType::Assignment,
        time: Utc::now(),
        worker: update_task_model.worker.clone().unwrap(),
        progress: Some(0.0),
    };
    task.updated = Utc::now();
    task.status = TaskStatus::Running;
    task.owner = update_task_model.worker.clone();
    task.deadline = Some(Utc::now() + Duration::seconds(update_task_model.task_timeout_in_secs.unwrap_or(10)));

    task.history.push(task_assignment_history);
    task
}

/// # Errors
/// Returns `ScyllaOperationsError`
fn validate_reset_operation(task: &Task) -> Result<(), ScyllaOperationsError> {
    if task.status != TaskStatus::Running {
        Err(ScyllaOperationsError::InvalidOperation(
            UpdateOperation::Reset,
            TaskStatus::Running,
            task.status.clone(),
        ))
    } else if task.deadline.is_none() {
        Err(ScyllaOperationsError::MandatoryFieldMissing("deadline".to_string(), UpdateOperation::Reset))
    } else if task.deadline.unwrap() >= Utc::now() {
        Err(ScyllaOperationsError::ValidationFailed(
            "deadline not yet passed for reset operation".to_string(),
        ))
    } else {
        Ok(())
    }
}

/// # Panics
/// if `owner` is missing. This function is supposed to be used after `validate_reset_task`. Which has required checks
fn prepare_reset_task(mut task: Task) -> Task {
    let task_timeout_history = TaskHistory {
        worker: task.owner.unwrap(),
        progress: Some(task.progress),
        typ: TaskHistoryType::Timeout,
        time: Utc::now(),
    };
    task.deadline = None;
    task.owner = None;
    task.progress = 0.0;
    task.status = TaskStatus::Ready;
    task.updated = Utc::now();
    let last_history_entry = task.history.last();

    if let Some(history_value) = last_history_entry {
        if history_value.typ != TaskHistoryType::Yield {
            task.history.push(task_timeout_history);
        }
    }
    task
}

/// # Arguments
/// public function to update task
/// # Example
/// /*********************************/
///    // Lease operation
///  `   let t = Task {
///         status: TaskStatus::Ready,
///         ..Task::default()
///     };
///     let utm = UpdateTaskModel {
///         operation: UpdateOperation::Lease,
///         rn: "123".to_string(),
///         worker: Some("worker1".to_string()),
///         error: None,
///         progress: None,
///         status: None,
///     };
///     let updated_task = request_handler(t, &utm, Duration::seconds(10)).unwrap();
///     assert_eq!(updated_task.status, TaskStatus::Running);
///     assert_eq!(updated_task.owner, Some("worker1".to_string()));`
///     /*********************************/
/// See `unit tests` for more examples
///  # Errors
/// Returns `ScyllaOperationsError` in case of invalid data.

pub fn request_handler(task: Task, update_task_model: &UpdateTaskModel) -> Result<Task, ScyllaOperationsError> {
    match update_task_model.operation {
        UpdateOperation::Status => {
            validate_status_operation(&task, update_task_model)?;
            Ok(prepare_status_task(task, update_task_model))
        }
        UpdateOperation::HeartBeat => {
            validate_heart_beat_operation(&task, update_task_model)?;
            Ok(prepare_heart_beat_task(task, update_task_model))
        }
        UpdateOperation::Yield => {
            validate_yield_operation(&task)?;
            Ok(prepare_yield_task(task))
        }
        UpdateOperation::Lease => {
            validate_lease_operation(&task, update_task_model)?;
            Ok(prepare_lease_task(task, update_task_model))
        }
        UpdateOperation::Reset => {
            validate_reset_operation(&task)?;
            Ok(prepare_reset_task(task))
        }
    }
}

#[cfg(test)]
mod tests;
