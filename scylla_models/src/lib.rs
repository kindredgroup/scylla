//! Common models used by most crates
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fmt::{Debug, Display, Formatter};

#[derive(Debug)]
pub struct AddTaskModel {
  pub rn: String,
  pub spec: Value,
  pub priority: i8,
  pub queue: String,
}

#[derive(Debug)]
pub struct UpdateTaskModel {
  pub rn: String,
  pub operation: UpdateOperation,
  pub status: Option<TaskStatus>,
  pub error: Option<TaskError>,
  pub worker: Option<String>,
  pub progress: Option<f32>,
}

#[derive(Debug)]
pub struct GetTaskModel {
  pub limit: Option<i32>,
  pub queue: Option<String>,
  pub worker: Option<String>,
  pub status: Option<TaskStatus>,
}
impl Default for GetTaskModel {
  fn default() -> Self {
    GetTaskModel {
      limit: Some(100),
      queue: None,
      worker: None,
      status: None,
    }
  }
}

#[derive(Debug, Serialize, Deserialize, Clone, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum UpdateOperation {
  Yield,
  HeartBeat,
  Status,
  Lease,
  Reset,
}
impl Display for UpdateOperation {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    write!(f, "{:?}", self)
  }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub enum TaskStatus {
  #[serde(rename = "ready")]
  Ready,
  #[serde(rename = "running")]
  Running,
  #[serde(rename = "completed")]
  Completed,
  #[serde(rename = "aborted")]
  Aborted,
  #[serde(rename = "cancelled")]
  Cancelled,
}

pub trait TaskStatusExt {
  fn allowed_transitions(&self) -> &[TaskStatus];
}

// Even though Ready can be moved to Running stage. That is handled through Lease Operation and not statusOperation
impl TaskStatusExt for TaskStatus {
  fn allowed_transitions(&self) -> &[TaskStatus] {
    match self {
      TaskStatus::Ready => &[TaskStatus::Cancelled],
      TaskStatus::Running => &[TaskStatus::Completed, TaskStatus::Cancelled, TaskStatus::Aborted],
      TaskStatus::Completed | TaskStatus::Aborted | TaskStatus::Cancelled => &[],
    }
  }
}

impl Display for TaskStatus {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    write!(f, "{:?}", self)
  }
}
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct TaskHistory {
  pub typ: TaskHistoryType,
  pub worker: String,
  pub progress: Option<f32>,
  pub time: DateTime<Utc>,
}
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub enum TaskHistoryType {
  #[serde(rename = "TaskAssignment")]
  Assignment,
  #[serde(rename = "TaskTimeout")]
  Timeout,
  #[serde(rename = "TaskYield")]
  Yield,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct TaskError {
  pub code: String,
  pub args: serde_json::Value,
  pub description: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Task {
  pub rn: String,
  pub spec: Value,
  pub status: TaskStatus,
  pub queue: String, // settlement
  pub progress: f32,
  pub priority: i8,
  pub created: DateTime<Utc>,
  pub updated: DateTime<Utc>,
  pub deadline: Option<DateTime<Utc>>,
  pub owner: Option<String>,
  pub errors: Vec<TaskError>,
  pub history: Vec<TaskHistory>,
}
impl Default for Task {
  fn default() -> Self {
    Self {
      rn: String::default(),
      spec: serde_json::Value::default(),
      status: TaskStatus::Ready,
      queue: String::default(),
      progress: 0.0,
      priority: 0,
      created: Utc::now(),
      updated: Utc::now(),
      deadline: None,
      owner: None,
      errors: Vec::default(),
      history: Vec::default(),
    }
  }
}
// $coverage:ignore-start
#[cfg(test)]
mod tests {

    use crate::*;

  #[test]
  fn add_task_model_debug() {
    let atm = AddTaskModel {
      priority: 2,
      queue: String::from("new model"),
      rn: String::from("1.2.3"),
      spec: serde_json::Value::default()
    };
    assert_eq!(format!("{:?}", atm), "AddTaskModel { rn: \"1.2.3\", spec: Null, priority: 2, queue: \"new model\" }");
  }
  #[test]
  fn update_task_model_debug() {
    let utm = UpdateTaskModel {
      error: None,
      operation: UpdateOperation::HeartBeat,
      progress: None,
      rn: String::from("1.2.3"),
      status: None,
      worker: None
    };
    assert_eq!(format!("{:?}", utm), "UpdateTaskModel { rn: \"1.2.3\", operation: HeartBeat, status: None, error: None, worker: None, progress: None }");
  }
  #[test]
  fn get_task_model_debug() {
    let gtm = GetTaskModel::default();
    //debug trait
    assert_eq!(format!("{:?}", gtm), "GetTaskModel { limit: Some(100), queue: None, worker: None, status: None }");
    // default
    let gtm_default = GetTaskModel::default();
    assert_eq!(gtm_default.limit, Some(100));
    assert_eq!(gtm_default.queue, None);
    assert_eq!(gtm_default.status, None);
    assert_eq!(gtm_default.worker, None);
  }
  #[test]
  fn update_operation() {
    // display trait
    assert_eq!(format!("Update Operation is {}", UpdateOperation::HeartBeat), "Update Operation is HeartBeat");
  }

  #[test]
  fn task_status() {
    // allowed transitions
    assert_eq!(TaskStatus::Aborted.allowed_transitions(), &[]);
    assert_eq!(TaskStatus::Completed.allowed_transitions(), &[]);
    assert_eq!(TaskStatus::Cancelled.allowed_transitions(), &[]);
    assert_eq!(TaskStatus::Ready.allowed_transitions(), &[TaskStatus::Cancelled]);
    assert_eq!(TaskStatus::Running.allowed_transitions(), &[TaskStatus::Completed, TaskStatus::Cancelled, TaskStatus::Aborted]);
    // display trait
    assert_eq!(format!("Task Status is {}", TaskStatus::Running), "Task Status is Running");

    assert_eq!(format!("{:?}", TaskStatus::Running), "Running");
    assert_ne!(TaskStatus::Running, TaskStatus::Ready);
    assert_eq!(TaskStatus::Running.clone(), TaskStatus::Running);
    assert_eq!(TaskStatus::Running == TaskStatus::Running, true);
    assert_eq!(serde_json::to_string(&TaskStatus::Running).unwrap(), "\"running\"");
    assert_eq!(serde_json::from_str::<TaskStatus>("\"running\"").unwrap(), TaskStatus::Running);
    assert_eq!(serde_json::from_str::<TaskStatus>("\"ready\"").unwrap(), TaskStatus::Ready);
    assert_eq!(serde_json::from_str::<TaskStatus>("\"aborted\"").unwrap(), TaskStatus::Aborted);
    assert_eq!(serde_json::from_str::<TaskStatus>("\"completed\"").unwrap(), TaskStatus::Completed);
    assert_eq!(serde_json::from_str::<TaskStatus>("\"cancelled\"").unwrap(), TaskStatus::Cancelled);
  }

  #[test]
  fn task_history() {
    // debug trait
    let t_now = Utc::now();
    assert_eq!(format!("{:?}", TaskHistory {
      progress: None,
      time: t_now.clone(),
      typ: TaskHistoryType::Assignment,
      worker: String::from("worker1")
    }), format!("TaskHistory {{ typ: Assignment, worker: \"worker1\", progress: None, time: {:?} }}", t_now));
    assert_eq!(format!("{:?}", TaskHistoryType::Assignment), "Assignment");
    assert_ne!(TaskHistoryType::Assignment, TaskHistoryType::Yield);
    assert_eq!(TaskHistoryType::Assignment.clone(), TaskHistoryType::Assignment);
    assert_eq!(TaskHistoryType::Assignment == TaskHistoryType::Assignment, true);
    assert_eq!(serde_json::to_string(&TaskHistoryType::Assignment).unwrap(), "\"TaskAssignment\"");
    assert_eq!(serde_json::from_str::<TaskHistoryType>("\"TaskAssignment\"").unwrap(), TaskHistoryType::Assignment);
    assert_eq!(serde_json::from_str::<TaskHistoryType>("\"TaskTimeout\"").unwrap(), TaskHistoryType::Timeout);
    assert_eq!(serde_json::from_str::<TaskHistoryType>("\"TaskYield\"").unwrap(), TaskHistoryType::Yield);
  }

  #[test]
  fn task() {
    let t_now = Utc::now();
    let t = Task {
      created: t_now,
      updated: t_now,
      ..Task::default()
    };
    // debug trait
    assert_eq!(format!("{:?}", t), format!("Task {{ rn: \"\", spec: Null, status: Ready, queue: \"\", progress: 0.0, priority: 0, created: {0:?}, updated: {0:?}, deadline: None, owner: None, errors: [], history: [] }}", t_now));
    // default()
    let t = Task {
      created: t_now,
      updated: t_now,
      ..Task::default()
    };
    assert_eq!(t, Task {
      rn: String::default(),
      spec: serde_json::Value::default(),
      status: TaskStatus::Ready,
      queue: String::default(),
      progress: 0.0,
      priority: 0,
      created: t_now,
      updated: t_now,
      deadline: None,
      owner: None,
      errors: Vec::default(),
      history: Vec::default(),
    })

  }
}
