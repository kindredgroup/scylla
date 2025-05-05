//! Scylla Operations
use crate::error::ScyllaOperationsError;
use crate::update_task::request_handler;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use scylla_models::{AddTaskModel, GetTaskModel, Task, UpdateTaskModel};

pub struct ScyllaOperations {}

impl ScyllaOperations {
    pub fn add_task_operation(add_task_model: &AddTaskModel) -> Task {
        Task {
            rn: add_task_model.rn.clone(),
            spec: add_task_model.spec.clone(),
            queue: add_task_model.queue.clone(),
            priority: add_task_model.priority,
            ..Task::default()
        }
    }
    /// # Errors
    /// Returns `ScyllaOperationsError`
    pub fn update_task_operation(update_task_model: &UpdateTaskModel, task_to_update: Task) -> Result<Task, ScyllaOperationsError> {
        request_handler(task_to_update, update_task_model)
    }
}
#[async_trait]
pub trait Persistence
where
    Self::PersistenceError: From<ScyllaOperationsError>,
{
    type PersistenceError;

    async fn insert(&self, task: Task) -> Result<Task, Self::PersistenceError>;
    async fn update(&self, task: Task) -> Result<Task, Self::PersistenceError>;
    async fn query(&self, get_task_model: &GetTaskModel) -> Result<Vec<Task>, Self::PersistenceError>;
    async fn query_by_rn(&self, rn: String) -> Result<Task, Self::PersistenceError>;
    async fn reset_batch(&self) -> Result<Vec<Task>, Self::PersistenceError>;

    async fn lease_batch(&self, queue: String, limit: i32, worker: String, task_timeout_in_secs: i64) -> Result<Vec<Task>, Self::PersistenceError>;
    async fn delete_batch(&self, retention_time_in_secs: i64) -> Result<u64, Self::PersistenceError>;
}

#[cfg(test)]
mod tests;
