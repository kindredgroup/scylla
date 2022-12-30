//! PG Manager used by external crates to deal with Database operations. `PGAdapter` is not accessible without `PGManager`
use crate::adapter::PgAdapter;
use crate::error::PgAdapterError;
use scylla_models::{AddTaskModel, GetTaskModel, Task, TaskError, TaskStatus, UpdateOperation, UpdateTaskModel};
use scylla_operations::task::{Persistence, ScyllaOperations};
use scylla_pg_core::config::PGConfig;
use scylla_pg_core::connection::get_pool;

pub struct PgManager {
    pg_adapter: Box<dyn Persistence<PersistenceError = PgAdapterError> + Send + Sync>,
}

impl PgManager {
    // $coverage:ignore-start
    /// Ignored from coverage because of real database interactions. covered as part of component tests
    /// # Errors
    /// Returns `PgAdapterError`
    pub fn from_config(config: &PGConfig) -> Result<Self, PgAdapterError> {
        let pool = get_pool(config)?;
        Ok(Self {
            pg_adapter: Box::new(PgAdapter { pool }),
        })
    }
    // $coverage:ignore-end
    /// # Errors
    /// Returns `PgAdapterError`
    pub async fn fetch_task(&self, rn: String) -> Result<Task, PgAdapterError> {
        self.pg_adapter.query_by_rn(rn).await
    }
    /// # Errors
    /// Returns `PgAdapterError`
    pub async fn insert_task(&self, atm: AddTaskModel) -> Result<Task, PgAdapterError> {
        let task = ScyllaOperations::add_task_operation(&atm);
        self.pg_adapter.insert(task).await
    }
    /// # Errors
    /// Returns `PgAdapterError`
    pub async fn fetch_tasks(&self, get_task_model: GetTaskModel) -> Result<Vec<Task>, PgAdapterError> {
        self.pg_adapter.query(&get_task_model).await
    }
    /// # Errors
    /// Returns `PgAdapterError`
    pub async fn lease_task(&self, rn: String, worker: String) -> Result<Task, PgAdapterError> {
        let update_task_model = UpdateTaskModel {
            rn,
            worker: Some(worker),
            status: None,
            progress: None,
            operation: UpdateOperation::Lease,
            error: None,
        };
        self.update_task(&update_task_model).await
    }
    /// # Errors
    /// Returns `PgAdapterError`
    pub async fn heartbeat_task(&self, rn: String, progress: Option<f32>) -> Result<Task, PgAdapterError> {
        let update_task_model = UpdateTaskModel {
            rn,
            worker: None,
            status: None,
            progress,
            operation: UpdateOperation::HeartBeat,
            error: None,
        };
        self.update_task(&update_task_model).await
    }
    /// # Errors
    /// Returns `PgAdapterError`
    pub async fn cancel_task(&self, rn: String) -> Result<Task, PgAdapterError> {
        let update_task_model = UpdateTaskModel {
            rn,
            worker: None,
            status: Some(TaskStatus::Cancelled),
            progress: None,
            operation: UpdateOperation::Status,
            error: None,
        };
        self.update_task(&update_task_model).await
    }
    /// # Errors
    /// Returns `PgAdapterError`
    pub async fn complete_task(&self, rn: String) -> Result<Task, PgAdapterError> {
        let update_task_model = UpdateTaskModel {
            rn,
            worker: None,
            status: Some(TaskStatus::Completed),
            progress: None,
            operation: UpdateOperation::Status,
            error: None,
        };
        self.update_task(&update_task_model).await
    }
    /// # Errors
    /// Returns `PgAdapterError`
    pub async fn abort_task(&self, rn: String, error: TaskError) -> Result<Task, PgAdapterError> {
        let update_task_model = UpdateTaskModel {
            rn,
            worker: None,
            status: Some(TaskStatus::Aborted),
            progress: None,
            operation: UpdateOperation::Status,
            error: Some(error),
        };
        self.update_task(&update_task_model).await
    }
    /// # Errors
    /// Returns `PgAdapterError`
    pub async fn yield_task(&self, rn: String) -> Result<Task, PgAdapterError> {
        let update_task_model = UpdateTaskModel {
            rn,
            worker: None,
            status: None,
            progress: None,
            operation: UpdateOperation::Yield,
            error: None,
        };
        self.update_task(&update_task_model).await
    }
    /// # Errors
    /// Returns `PgAdapterError`
    pub async fn reset_task(&self, rn: String) -> Result<Task, PgAdapterError> {
        let update_task_model = UpdateTaskModel {
            rn,
            worker: None,
            status: None,
            progress: None,
            operation: UpdateOperation::Reset,
            error: None,
        };
        self.update_task(&update_task_model).await
    }
    /// # Errors
    /// Returns `PgAdapterError`
    async fn update_task(&self, utm: &UpdateTaskModel) -> Result<Task, PgAdapterError> {
        let task_to_update = self.fetch_task(utm.rn.clone()).await?;
        let task = ScyllaOperations::update_task_operation(utm, task_to_update)?;
        self.pg_adapter.update(task).await
    }
}

#[cfg(test)]
mod tests;
