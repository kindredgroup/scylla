use scylla_models::TaskStatus;
use scylla_pg_lib::error::PgAdapterError;

#[derive(Debug, thiserror::Error)]
pub enum JSScyllaError {
    #[error("Validation failed for fields: {0}")]
    ArgumentValidationError(String),
    // #[error("Error: {0}")]
    // DomainError(#[from] PgAdapterError),
}

pub fn validate_status(status: String) -> Result<TaskStatus, JSScyllaError> {
    match status.as_str() {
        "running" => Ok(TaskStatus::Running),
        "ready" => Ok(TaskStatus::Ready),
        "completed" => Ok(TaskStatus::Completed),
        "cancelled" => Ok(TaskStatus::Cancelled),
        "aborted" => Ok(TaskStatus::Aborted),
        _ => Err(JSScyllaError::ArgumentValidationError("Invalid Task Status".to_string())),
    }
}

pub fn validate_json(spec: String, field: String) -> Result<serde_json::Value, JSScyllaError> {
    match serde_json::from_str(spec.as_str()) {
        Ok(t) => Ok(t),
        Err(_) => Err(JSScyllaError::ArgumentValidationError(format!("Invalid JSON for {field}"))),
    }
}

// pub fn validate_progress(progress: f64) -> Result<f32, JSScyllaError>{
//   return match f32::try_from(progress) {
//     Ok(t) => Ok(t),
//     Err(_) => Err(JSScyllaError::ArgumentValidationError("Invalid value for progress".to_string()))
//   }
// }

pub fn validate_port(port: u32) -> Result<u16, JSScyllaError> {
    match u16::try_from(port) {
        Ok(t) => Ok(t),
        Err(_) => Err(JSScyllaError::ArgumentValidationError("Invalid value for port".to_string())),
    }
}
