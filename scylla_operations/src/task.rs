//! Scylla Operations
use async_trait::async_trait;
use crate::update_task::request_handler;
use crate::error::ScyllaOperationsError;
use scylla_models::*;
use chrono::Duration;

#[async_trait]
pub trait ScyllaOperations where Self::PersistenceError: From<ScyllaOperationsError> {
  type PersistenceError;

  async fn insert(&self, task: Task) -> Result<Task, Self::PersistenceError>;
  async fn update(&self, task: Task) -> Result<Task, Self::PersistenceError>;
  async fn query(&self, get_task_model: &GetTaskModel) -> Result<Vec<Task>, Self::PersistenceError>;
  async fn query_by_rn(&self, rn: String) -> Result<Task, Self::PersistenceError>;

  async fn add_task(&self, add_task_model: &AddTaskModel) -> Result<Task, Self::PersistenceError> {
    let task = Task {
      rn: add_task_model.rn.clone(),
      spec: add_task_model.spec.clone(),
      queue: add_task_model.queue.clone(),
      priority: add_task_model.priority,
      ..Task::default()
    };
    self.insert(task).await.map_err(|e| e.into())
  }
  async fn update_task(&self, update_task_model: &UpdateTaskModel) -> Result<Task, Self::PersistenceError> {
    let task_to_update = self.get_task(update_task_model.rn.clone()).await?;
    let task = request_handler(task_to_update, update_task_model, Duration::seconds(1))?;
    self.update(task).await.map_err(|e| e.into())
  }
  async fn get_tasks(&self, get_task_model: &GetTaskModel) -> Result<Vec<Task>, Self::PersistenceError> {
    self.query(get_task_model).await.map_err(|e| e.into())
  }
  async fn get_task(&self, rn: String) -> Result<Task, Self::PersistenceError> {
    self.query_by_rn(rn).await.map_err(|e| e.into())
  }
}

#[cfg(test)]
mod tests;
