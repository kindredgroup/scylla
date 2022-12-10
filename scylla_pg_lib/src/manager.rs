use scylla_models::*;
use crate::adapter::PgAdapter;
use crate::error::PgAdapterError;
use scylla_operations::task::ScyllaOperations;
use scylla_pg_core::connection::get_pool;
use scylla_pg_core::config::PGConfig;

// pub struct PgManager {
//     pg_adapter: Box<dyn ScyllaOperations<PersistenceError = PgAdapterError> + Sync + Send>
// }

pub struct PgManager {
  pg_adapter: PgAdapter
}

impl PgManager {
  pub fn from_config(config: PGConfig) -> Result<Self, PgAdapterError> {
    let pool = get_pool(config)?;
    Ok(Self { pg_adapter: PgAdapter {pool} } )
  }
  pub async fn fetch_task(&self, rn: String) -> Result<Task, PgAdapterError> {
    let task = self.pg_adapter.get_task(rn).await?;
    Ok(task)
  }
  pub async fn insert_task(&self, atm: AddTaskModel) -> Result<Task, PgAdapterError> {
    let task =  self.pg_adapter.add_task(&atm).await?;
    Ok(task)
  }
  pub async fn fetch_tasks(&self, get_task_model: GetTaskModel) -> Result<Vec<Task>, PgAdapterError> {
    let tasks = self.pg_adapter.get_tasks(&get_task_model).await?;
    Ok(tasks)
  }
  pub async fn lease_task(&self, rn: String, worker: String) -> Result<Task, PgAdapterError> {
    let update_task_model = UpdateTaskModel {
      rn,
      worker: Some(worker),
      status: None,
      progress: None,
      operation: UpdateOperation::Lease,
      error: None,
    };
    let task = self.pg_adapter.update_task(&update_task_model).await?;
    Ok(task)
  }
  pub async fn heartbeat_task(&self, rn: String, progress: Option<f32>) -> Result<Task, PgAdapterError> {
    let update_task_model = UpdateTaskModel {
      rn,
      worker: None,
      status: None,
      progress,
      operation: UpdateOperation::HeartBeat,
      error: None,
    };
    let task = self.pg_adapter.update_task(&update_task_model).await?;
    Ok(task)
  }
  pub async fn cancel_task(&self, rn: String) -> Result<Task, PgAdapterError> {
    let update_task_model = UpdateTaskModel {
      rn,
      worker: None,
      status: Some(TaskStatus::Cancelled),
      progress: None,
      operation: UpdateOperation::Status,
      error: None,
    };
    let task = self.pg_adapter.update_task(&update_task_model).await?;
    Ok(task)
  }
  pub async fn complete_task(&self, rn: String) -> Result<Task, PgAdapterError> {
    let update_task_model = UpdateTaskModel {
      rn,
      worker: None,
      status: Some(TaskStatus::Completed),
      progress: None,
      operation: UpdateOperation::Status,
      error: None,
    };
    let task = self.pg_adapter.update_task(&update_task_model).await?;
    Ok(task)
  }
  pub async fn abort_task(&self, rn: String, error: TaskError) -> Result<Task, PgAdapterError> {
    let update_task_model = UpdateTaskModel {
      rn,
      worker: None,
      status: Some(TaskStatus::Aborted),
      progress: None,
      operation: UpdateOperation::Status,
      error: Some(error),
    };
    let task = self.pg_adapter.update_task(&update_task_model).await?;
    Ok(task)
  }
  pub async fn yield_task(&self, rn: String) -> Result<Task, PgAdapterError> {
    let update_task_model = UpdateTaskModel {
      rn,
      worker: None,
      status: None,
      progress: None,
      operation: UpdateOperation::Yield,
      error: None,
    };
    let task = self.pg_adapter.update_task(&update_task_model).await?;
    Ok(task)
  }
  pub async fn reset_task(&self, rn: String) -> Result<Task, PgAdapterError> {
    let update_task_model = UpdateTaskModel {
      rn,
      worker: None,
      status: None,
      progress: None,
      operation: UpdateOperation::Reset,
      error: None,
    };
    let task = self.pg_adapter.update_task(&update_task_model).await?;
    Ok(task)
  }
}

// #[cfg(test)]
// mod tests;
