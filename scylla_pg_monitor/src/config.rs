// use log::info;
use serde::{Deserialize, Serialize};

use std::fmt::Debug;

use config::ConfigError;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Eq)]
pub struct Config {
  // Postgres
  pub pghost: String,
  pub pgport: u16,
  pub pguser: String,
  pub pgpassword: String,
  pub pgdatabase: String,
  // pub admin_pg_user: String,
  // pub admin_pg_password: String,
  // pub admin_pg_database: String,

  // Monitor Config
  // pub monitor_interval_in_secs: u64,
  // pub default_get_tasks_limit: i32,
  // pub task_time_out_in_secs: i64,
}

impl Config {
  pub fn from_env() -> Result<Self, ConfigError> {
    // info!("Loading config from environment variables");

    config::Config::builder()
      .add_source(config::Environment::default())
      .build()
      .unwrap()
      .try_deserialize()
  }

  pub fn to_pg_config(&self) -> tokio_postgres::Config {
    let mut pg_config = tokio_postgres::Config::new();
    pg_config
      .host(&self.pghost)
      .port(self.pgport)
      .user(&self.pguser)
      .password(&self.pgpassword)
      .dbname(&self.pgdatabase);
    pg_config
  }
  pub fn to_without_db_config(&self) -> tokio_postgres::Config {
    let mut pg_config = tokio_postgres::Config::new();
    pg_config
      .host(&self.pghost)
      .port(self.pgport)
      .user(&self.pguser)
      .password(&self.pgpassword);
    pg_config
  }
  // pub fn to_pg_admin_config(&self) -> tokio_postgres::Config {
  //   let mut pg_config = tokio_postgres::Config::new();
  //
  //   pg_config
  //     .host(&self.pghost)
  //     .port(self.pgport)
  //     .user(&self.admin_pg_user)
  //     .password(&self.admin_pg_password)
  //     .dbname(&self.admin_pg_database);
  //   pg_config
  // }
}

impl Default for Config {
  fn default() -> Self {
    Config {
      // Postgres
      pghost: "".to_owned(),
      pgport: 1234,
      pguser: "".to_owned(),
      pgpassword: "".to_owned(),
      pgdatabase: "".to_owned(),
      // admin_pg_user: "".to_owned(),
      // admin_pg_database: "".to_owned(),
      // admin_pg_password: "".to_owned(),
      // Monitor
      // monitor_interval_in_secs: 0,
      // default_get_tasks_limit: 0,
      // task_time_out_in_secs: 0,
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use std::env;

  #[test]
  fn read_env_to_config() {
    env::set_var("PGHOST", "localhost");
    env::set_var("PGPORT", "5432");
    env::set_var("PGUSER", "pgadmin");
    env::set_var("PGPASSWORD", "pgpass");
    env::set_var("PGDATABASE", "db");
    // env::set_var("ADMIN_PG_USER", "admin");
    // env::set_var("ADMIN_PG_PASSWORD", "admin");
    // env::set_var("ADMIN_PG_DATABASE", "postgres");
    // env::set_var("MONITOR_INTERVAL_IN_SECS", "10");
    // env::set_var("DEFAULT_GET_TASKS_LIMIT", "100");
    // env::set_var("TASK_TIME_OUT_IN_SECS", "10");

    // Initialize config from environment variables
    let config = Config::from_env().unwrap();
    assert_eq!(
      config,
      Config {
        pghost: "localhost".to_owned(),
        pgport: 5432,
        pguser: "pgadmin".to_owned(),
        pgpassword: "pgpass".to_owned(),
        pgdatabase: "db".to_owned(),
        // admin_pg_password: "admin".to_owned(),
        // admin_pg_database: "postgres".to_owned(),
        // admin_pg_user: "admin".to_owned(),
        // monitor_interval_in_secs: 10,
        // default_get_tasks_limit: 100,
        // task_time_out_in_secs: 10
      }
    );
  }
}
