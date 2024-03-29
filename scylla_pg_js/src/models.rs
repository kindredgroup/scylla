// $coverage:ignore-start
use napi_derive::napi;

#[napi(object)]
pub struct JsAddTaskModel {
    pub rn: String,
    pub spec: String,
    pub priority: i8,
    pub queue: String,
}
#[napi(object)]
pub struct JsGetTasksModel {
    pub worker: Option<String>,
    pub status: Option<String>,
    pub limit: Option<i32>,
    pub queue: Option<String>,
}
#[napi(object)]
pub struct JsTaskError {
    pub code: String,
    pub args: String,
    pub description: String,
}
