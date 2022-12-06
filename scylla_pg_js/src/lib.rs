mod models;
mod validator;
use napi::sys::{napi_env, napi_value};
use napi::{NapiRaw, NapiValue};
use napi_derive::napi;
use scylla_models::{AddTaskModel, GetTaskModel, TaskError};
use scylla_pg_lib::manager::{DbConfig, PgManager};
use std::fmt::Display;

use models::*;
use validator::*;

#[napi(object)]
pub struct JsDbConfig {
    pub host: String,
    pub port: u32,
    pub user: String,
    pub password: String,
    pub db_name: String,
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
    // #[napi(constructor)]
    // pub async fn new(js_db_config: JsDbConfig) -> Self {
    //   Self { pg_manager: None }
    // }
    #[napi]
    pub async fn init_pg_config(js_db_config: JsDbConfig) -> napi::Result<ScyllaManager> {
        let port = validate_port(js_db_config.port)?;
        let db_config = DbConfig {
            host: js_db_config.host,
            port,
            user: js_db_config.user,
            password: js_db_config.password,
            db_name: js_db_config.db_name,
        };
        println!("pool initiated");
        Ok(Self {
            pg_manager: PgManager::from_config(db_config).map_err(map_error_to_napi_error)?,
        })
    }
    #[napi]
    pub async fn get_task(&self, rn: String) -> napi::Result<String> {
        let task_result = self.pg_manager.fetch_task(rn).await;
        map_lib_response!(task_result)
    }
    #[napi]
    pub async fn get_tasks(&self, js_gtm: JsGetTasksModel) -> napi::Result<String> {
        let status_val = match js_gtm.status {
            None => None,
            Some(status) => Some(validate_status(status)?),
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

    #[napi]
    pub async fn add_task(&self, js_atm: JsAddTaskModel) -> napi::Result<String> {
        let spec = validate_json(js_atm.spec, "spec".to_string())?;
        let atm = AddTaskModel {
            rn: js_atm.rn,
            priority: js_atm.priority,
            spec,
            queue: js_atm.queue,
        };
        let task_result = self.pg_manager.insert_task(atm).await;
        map_lib_response!(task_result)
    }

    #[napi]
    pub async fn lease_task(&self, rn: String, worker: String) -> napi::Result<String> {
        let task_result = self.pg_manager.lease_task(rn, worker).await;
        map_lib_response!(task_result)
    }

    #[napi]
    pub async fn yield_task(&self, rn: String) -> napi::Result<String> {
        let task_result = self.pg_manager.yield_task(rn).await;
        map_lib_response!(task_result)
    }

    #[napi]
    pub async fn complete_task(&self, rn: String) -> napi::Result<String> {
        let task_result = self.pg_manager.complete_task(rn).await;
        map_lib_response!(task_result)
    }

    #[napi]
    pub async fn cancel_task(&self, rn: String) -> napi::Result<String> {
        let task_result = self.pg_manager.cancel_task(rn).await;
        map_lib_response!(task_result)
    }

    #[napi]
    pub async fn abort_task(&self, rn: String, js_error: JsTaskError) -> napi::Result<String> {
        let error_args = validate_json(js_error.args, "errors.args".to_string())?;
        let task_error = TaskError {
            code: js_error.code,
            args: error_args,
            description: js_error.description,
        };
        let task_result = self.pg_manager.abort_task(rn, task_error).await;
        map_lib_response!(task_result)
    }

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

// impl NapiRaw for DBManager {
//   unsafe fn raw(&self) -> napi_value {
//     todo!()
//   }
// }
//
// impl NapiValue for DBManager {
//   unsafe fn from_raw(env: napi_env, value: napi_value) -> napi::Result<Self> {
//     todo!()
//   }
//
//   unsafe fn from_raw_unchecked(env: napi_env, value: napi_value) -> Self {
//     todo!()
//   }
// }

fn map_error_to_napi_error<T: Display>(e: T) -> napi::Error {
    napi::Error::from_reason(e.to_string())
}

impl From<JSScyllaError> for napi::Error {
    fn from(scylla_error: JSScyllaError) -> Self {
        map_error_to_napi_error(scylla_error)
    }
}

// #[napi]
// async fn init_pg_config(js_db_config: JsDbConfig) -> napi::Result<DBManager> {
//   let port = validate_port(js_db_config.port)?;
//   let db_config = DbConfig {
//     host: js_db_config.host,
//     port,
//     user: js_db_config.user,
//     password: js_db_config.password,
//     db_name: js_db_config.db_name,
//   };
//   Ok(DBManager {
//     pg_manager: PgManager::from_config(db_config).map_err(map_error_to_napi_error)?,
//   })
// }

// #[napi]
// async fn get_task<T>(dbm: DBManager, rn: String) -> napi::Result<String> {
//   let task_result = dbm.pg_manager.fetch_task(rn).await;
//   map_lib_response!(task_result)
// }
// #[napi(object)]
// pub struct JsGetTasksModel {
//   pub worker: Option<String>,
//   pub status: Option<String>,
//   pub limit: Option<i32>,
//   pub queue: Option<String>,
// }
//
// #[napi]
// async fn get_tasks(js_gtm: JsGetTasksModel) -> napi::Result<String> {
//   let status_val = match js_gtm.status {
//     None => None,
//     Some(status) => Some(validate_status(status)?),
//   };
//
//   let gtm = GetTaskModel {
//     worker: js_gtm.worker,
//     status: status_val,
//     limit: js_gtm.limit,
//     queue: js_gtm.queue,
//   };
//   let task_result = get_tasks_from_db(gtm).await;
//   map_lib_response!(task_result)
// }
// #[napi(object)]
// pub struct JsAddTaskModel {
//   pub rn: String,
//   pub spec: String,
//   pub priority: i8,
//   pub queue: String,
// }
//
// #[napi]
// async fn add_task(js_atm: JsAddTaskModel) -> napi::Result<String> {
//   let spec = validate_json(js_atm.spec, "spec".to_string())?;
//   let atm = AddTaskModel {
//     rn: js_atm.rn,
//     priority: js_atm.priority,
//     spec,
//     queue: js_atm.queue,
//   };
//   let task_result = add_task_db(atm).await;
//   map_lib_response!(task_result)
// }
//
// #[napi]
// async fn lease_task(rn: String, worker: String) -> napi::Result<String> {
//   let task_result = lease_task_db(rn, worker).await;
//   map_lib_response!(task_result)
// }
//
// #[napi]
// async fn yield_task(rn: String) -> napi::Result<String> {
//   let task_result = yield_task_db(rn).await;
//   map_lib_response!(task_result)
// }
//
// #[napi]
// async fn complete_task(rn: String) -> napi::Result<String> {
//   let task_result = complete_task_db(rn).await;
//   map_lib_response!(task_result)
// }
//
// #[napi]
// async fn cancel_task(rn: String) -> napi::Result<String> {
//   let task_result = cancel_task_db(rn).await;
//   map_lib_response!(task_result)
// }
// #[napi(object)]
// pub struct JsTaskError {
//   pub code: String,
//   pub args: String,
//   pub description: String,
// }
//
// #[napi]
// async fn abort_task(rn: String, js_error: JsTaskError) -> napi::Result<String> {
//   let error_args = validate_json(js_error.args, "errors.args".to_string())?;
//   let task_error = TaskError {
//     code: js_error.code,
//     args: error_args,
//     description: js_error.description,
//   };
//   let task_result = abort_task_db(rn, task_error).await;
//   map_lib_response!(task_result)
// }
//
// #[napi]
// async fn heart_beat_task(rn: String, progress: Option<f64>) -> napi::Result<String> {
//   let mut progress_value = None;
//   if let Some(p) = progress {
//     progress_value = Some(p as f32);
//   }
//   let task_result = heartbeat_task_db(rn, progress_value).await;
//   map_lib_response!(task_result)
// }
