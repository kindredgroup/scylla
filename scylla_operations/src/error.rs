use scylla_models::*;

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum ScyllaOperationsError {
  #[error("Invalid status passed. Current task status is {0:?}. Allowed status transitions are {1:?}")]
  InvalidStatusTransition(TaskStatus, Vec<TaskStatus>),
  #[error("Task is in terminal status: {0:?}. It should be in one of the non-terminal status {1:?}.")]
  TerminalTaskStatus(TaskStatus, Vec<TaskStatus>),
  #[error("Mandatory field missing {0} for operation {1:?}")]
  MandatoryFieldMissing(String, UpdateOperation),
  #[error("Invalid Operation: {0:?}. Task should be in {1:?} status. Current task status is {2:?}.")]
  InvalidOperation(UpdateOperation, TaskStatus, TaskStatus),
  #[error("Validation failed: {0}")]
  ValidationFailed(String),
}
// $coverage:ignore-start
#[cfg(test)]
mod tests {
  #[test]
fn scylla_operations_error_print_checks() {
  use crate::error::ScyllaOperationsError;
  use scylla_models::*;

  assert_eq!(ScyllaOperationsError::InvalidStatusTransition(TaskStatus::Ready, vec![TaskStatus::Running, TaskStatus::Aborted]).to_string(), "Invalid status passed. Current task status is Ready. Allowed status transitions are [Running, Aborted]".to_string());
  assert_eq!(ScyllaOperationsError::TerminalTaskStatus(TaskStatus::Ready, vec![TaskStatus::Running, TaskStatus::Aborted]).to_string(), "Task is in terminal status: Ready. It should be in one of the non-terminal status [Running, Aborted].".to_string());
  assert_eq!(ScyllaOperationsError::MandatoryFieldMissing("error".to_string(), UpdateOperation::Lease).to_string(), "Mandatory field missing error for operation Lease".to_string());
  assert_eq!(ScyllaOperationsError::InvalidOperation(UpdateOperation::Yield, TaskStatus::Completed, TaskStatus::Running).to_string(), "Invalid Operation: Yield. Task should be in Completed status. Current task status is Running.".to_string());
  assert_eq!(ScyllaOperationsError::ValidationFailed("field validation failed".to_string()).to_string(), "Validation failed: field validation failed".to_string());
}

}

