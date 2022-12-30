// $coverage:ignore-start
mod models;
mod validator;
use napi_derive::napi;
use scylla_models::{AddTaskModel, GetTaskModel, TaskError};
use scylla_pg_core::config::PGConfig;
use scylla_pg_lib::manager::PgManager;
use std::fmt::Display;

use models::{JsAddTaskModel, JsGetTasksModel, JsTaskError};
use validator::{validate_json, validate_port, validate_status, JSScyllaError};

#[napi(object)]
pub struct JsDbConfig {
    pub pg_host: String,
    pub pg_port: u32,
    pub pg_user: String,
    pub pg_password: String,
    pub pg_database: String,
}
macro_rules! map_lib_response {
    ($task_result: expr) => {
        match $task_result {
            Ok(t) => Ok(serde_json::to_string(&t).unwrap()),
            Err(e) => Err(napi::Error::from_reason(e.to_string())),
        }
    };
}
#[napi]
pub struct ScyllaManager {
    pg_manager: PgManager,
}

#[napi]
impl ScyllaManager {
    /// # Errors
    /// Convert rust error into `napi::Error`
    #[napi]
    pub fn init_pg_config(js_db_config: JsDbConfig) -> napi::Result<ScyllaManager> {
        let port = validate_port(js_db_config.pg_port)?;
        let pg_config = PGConfig {
            pg_host: js_db_config.pg_host,
            pg_port: port,
            pg_user: js_db_config.pg_user,
            pg_password: js_db_config.pg_password,
            pg_database: js_db_config.pg_database,
        };
        Ok(Self {
            pg_manager: PgManager::from_config(&pg_config).map_err(map_error_to_napi_error)?,
        })
    }
    /// # Errors
    /// Convert rust error into `napi::Error`
    #[napi]
    pub async fn get_task(&self, rn: String) -> napi::Result<String> {
        let task_result = self.pg_manager.fetch_task(rn).await;
        map_lib_response!(task_result)
    }
    /// # Errors
    /// Convert rust error into `napi::Error`
    #[napi]
    pub async fn get_tasks(&self, js_gtm: JsGetTasksModel) -> napi::Result<String> {
        let status_val = match js_gtm.status {
            None => None,
            Some(status) => Some(validate_status(status.as_str())?),
        };

        let gtm = GetTaskModel {
            worker: js_gtm.worker,
            status: status_val,
            limit: js_gtm.limit,
            queue: js_gtm.queue,
        };
        let task_result = self.pg_manager.fetch_tasks(gtm).await;
        map_lib_response!(task_result)
    }
    /// # Errors
    /// Convert rust error into `napi::Error`
    #[napi]
    pub async fn add_task(&self, js_atm: JsAddTaskModel) -> napi::Result<String> {
        let spec = validate_json(js_atm.spec.as_str(), "spec")?;
        let atm = AddTaskModel {
            rn: js_atm.rn,
            priority: js_atm.priority,
            spec,
            queue: js_atm.queue,
        };
        let task_result = self.pg_manager.insert_task(atm).await;
        map_lib_response!(task_result)
    }
    /// # Errors
    /// Convert rust error into `napi::Error`
    #[napi]
    pub async fn lease_task(&self, rn: String, worker: String) -> napi::Result<String> {
        let task_result = self.pg_manager.lease_task(rn, worker).await;
        map_lib_response!(task_result)
    }
    /// # Errors
    /// Convert rust error into `napi::Error`
    #[napi]
    pub async fn yield_task(&self, rn: String) -> napi::Result<String> {
        let task_result = self.pg_manager.yield_task(rn).await;
        map_lib_response!(task_result)
    }
    /// # Errors
    /// Convert rust error into `napi::Error`
    #[napi]
    pub async fn complete_task(&self, rn: String) -> napi::Result<String> {
        let task_result = self.pg_manager.complete_task(rn).await;
        map_lib_response!(task_result)
    }
    /// # Errors
    /// Convert rust error into `napi::Error`
    #[napi]
    pub async fn cancel_task(&self, rn: String) -> napi::Result<String> {
        let task_result = self.pg_manager.cancel_task(rn).await;
        map_lib_response!(task_result)
    }
    /// # Errors
    /// Convert rust error into `napi::Error`
    #[napi]
    pub async fn abort_task(&self, rn: String, js_error: JsTaskError) -> napi::Result<String> {
        let error_args = validate_json(js_error.args.as_str(), "errors.args")?;
        let task_error = TaskError {
            code: js_error.code,
            args: error_args,
            description: js_error.description,
        };
        let task_result = self.pg_manager.abort_task(rn, task_error).await;
        map_lib_response!(task_result)
    }
    /// # Errors
    /// Convert rust error into `napi::Error`
    #[napi]
    pub async fn heart_beat_task(&self, rn: String, progress: Option<f64>) -> napi::Result<String> {
        let mut progress_value = None;
        if let Some(p) = progress {
            progress_value = Some(p as f32);
        }
        let task_result = self.pg_manager.heartbeat_task(rn, progress_value).await;
        map_lib_response!(task_result)
    }
}
/// # Errors
/// Convert rust error into `napi::Error`
fn map_error_to_napi_error<T: Display>(e: T) -> napi::Error {
    napi::Error::from_reason(e.to_string())
}

impl From<JSScyllaError> for napi::Error {
    fn from(scylla_error: JSScyllaError) -> Self {
        map_error_to_napi_error(scylla_error)
    }
}
