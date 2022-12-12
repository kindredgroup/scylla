

use std::fmt::Display;

use deadpool_postgres::{PoolError, BuildError};
use scylla_operations::error::ScyllaOperationsError;
#[derive(Debug)]
pub enum PgAdapterError {
  ScyllaOpsError(ScyllaOperationsError),
  PoolCreationError(BuildError),
  PoolError(PoolError),
  DbError(tokio_postgres::Error),
  DuplicateTask(String),
  NoTaskFound(String)
}

impl From<ScyllaOperationsError> for PgAdapterError {
  fn from(scylla_op_error: ScyllaOperationsError) -> Self {
    Self::ScyllaOpsError(scylla_op_error)
  }
}
impl From<PoolError> for PgAdapterError {
  fn from(pool_error: PoolError) -> Self {
    Self::PoolError(pool_error)
  }
}
impl From<tokio_postgres::Error> for PgAdapterError {
  fn from(pg_error: tokio_postgres::Error) -> Self {
    Self::DbError(pg_error)
  }
}
impl From<BuildError> for PgAdapterError {
  fn from(build_error: BuildError) -> Self {
    Self::PoolCreationError(build_error)
  }
}


impl Display for PgAdapterError{
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
      match self {
        PgAdapterError::DbError(pg) => write!(f, "{pg}"),
        PgAdapterError::DuplicateTask(rn) => write!(f, "Task already exist for {rn}"),
        PgAdapterError::NoTaskFound(rn) => write!(f, "No task found for {rn}"),
        PgAdapterError::PoolCreationError(build_error) => write!(f, "{build_error}"),
        PgAdapterError::PoolError(pool_error) => write!(f, "{pool_error}"),
        PgAdapterError::ScyllaOpsError(sc_ops_error) => write!(f, "{sc_ops_error}")
      }
  }
}
// $coverage:ignore-start
#[cfg(test)]
mod tests {

use scylla_operations::error::ScyllaOperationsError;
use crate:: error::PgAdapterError;

  #[test]
fn scylla_operations_error_print_checks() {

  assert_eq!(PgAdapterError::ScyllaOpsError(ScyllaOperationsError::ValidationFailed("Field validation failed".to_string())).to_string(),"Validation failed: Field validation failed".to_string());
  assert_eq!(PgAdapterError::DuplicateTask("sample".to_string()).to_string(),"Task already exist for sample".to_string());
  assert_eq!(PgAdapterError::NoTaskFound("sample".to_string()).to_string(),"No task found for sample".to_string());
  assert_eq!(format!("{:?}",PgAdapterError::DuplicateTask("sample".to_string())), "DuplicateTask(\"sample\")".to_string());
}

}