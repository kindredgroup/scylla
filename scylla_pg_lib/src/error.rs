

use std::fmt::Display;

use deadpool_postgres::{PoolError, BuildError};
use scylla_operations::error::ScyllaOperationsError;
#[derive(Debug)]
pub enum PgAdapterError {
  PoolNotValid,
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
  fn from(pool_error: tokio_postgres::Error) -> Self {
    Self::DbError(pool_error)
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
        PgAdapterError::PoolNotValid => write!(f, "Pool instance not valid. Consider creating a new instance"),
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

  // assert_eq!(PgAdapterError::PoolNotValid.to_string(),"Db pool is not valid.".to_string());
  // assert_eq!(PgAdapterError::ScyllaOpsError(ScyllaOperationsError::ValidationFailed("Field validation failed".to_string())).to_string(),"Validation failed: Field validation failed".to_string());
  // assert_eq!(PgAdapterError::PoolCreationError("sample".to_string()).to_string(),"Pool creation error: sample".to_string());
  // assert_eq!(PgAdapterError::DbError(deadpool_postgres::PoolError::Closed).to_string(),"Pool start error: Pool has been closed".to_string() );
}

// #[test]
// fn from_scylla_ops_error_implemented() {
//   fn returns_pg_adapter_error () -> PgAdapterError {
//     ScyllaOperationsError::ValidationFailed("Field validation failed".to_string()).into()
//   }
//   //string comaprison
//   assert_eq!(returns_pg_adapter_error().to_string(), PgAdapterError::ScyllaOpsError(ScyllaOperationsError::ValidationFailed("Field validation failed".to_string())).to_string());
//   fn compare_pg_adapter_error (pga: PgAdapterError) -> PgAdapterError {
//     match pga {
//       PgAdapterError::PoolCreationError(_s) => PgAdapterError::PoolCreationError("b".to_string()),
//       PgAdapterError::PoolNotValid => PgAdapterError::PoolNotValid,
//       PgAdapterError::ScyllaOpsError(sc) =>  PgAdapterError::ScyllaOpsError(sc),
//       PgAdapterError::DbError(x) => PgAdapterError::DbError(x)
//     }
//   }
//   assert_eq!(compare_pg_adapter_error(PgAdapterError::PoolNotValid).to_string(), PgAdapterError::PoolNotValid.to_string());  
// }

// #[test]
// fn from_pool_error_implemented() {
//   fn convert_error(f: fn() -> deadpool_postgres::PoolError) -> PgAdapterError {
//     f().into()
//   }
//   assert_eq!(convert_error(|| deadpool_postgres::PoolError::Closed).to_string(), PgAdapterError::DbError(deadpool_postgres::PoolError::Closed).to_string());
// }
}