// use log::info;
use serde::{Deserialize, Serialize};

use std::fmt::Debug;

use config::{ConfigError, Config};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Eq)]
pub struct PGConfig {

  pub pg_host: String,
  pub pg_port: u16,
  pub pg_user: String,
  pub pg_password: String,
  pub pg_database: String,
}

impl PGConfig {
  pub fn from_env() -> Result<Self, ConfigError> {
    // info!("Loading config from environment variables");

    Config::builder()
      .add_source(config::Environment::default())
      .build()
      .unwrap()
      .try_deserialize()
  }

  pub fn to_pg_config(&self) -> tokio_postgres::Config {
    let mut pg_config = tokio_postgres::Config::new();
    pg_config
      .host(&self.pg_host)
      .port(self.pg_port)
      .user(&self.pg_user)
      .password(&self.pg_password)
      .dbname(&self.pg_database);
    pg_config
  }
  pub fn to_without_db_config(&self) -> tokio_postgres::Config {
    let mut pg_config = tokio_postgres::Config::new();
    pg_config
      .host(&self.pg_host)
      .port(self.pg_port)
      .user(&self.pg_user)
      .password(&self.pg_password);
    pg_config
  }
}

impl Default for PGConfig {
  fn default() -> Self {
    PGConfig {
      pg_host: "".to_owned(),
      pg_port: 1234,
      pg_user: "".to_owned(),
      pg_password: "".to_owned(),
      pg_database: "".to_owned(),
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use std::env;

  #[test]
  fn read_env_to_config() {
    env::set_var("PG_HOST", "localhost");
    env::set_var("PG_PORT", "5432");
    env::set_var("PG_USER", "pgadmin");
    env::set_var("PG_PASSWORD", "pgpass");
    env::set_var("PG_DATABASE", "db");
    let config = PGConfig::from_env().unwrap();
    assert_eq!(
      config,
      PGConfig {
        pg_host: "localhost".to_owned(),
        pg_port: 5432,
        pg_user: "pgadmin".to_owned(),
        pg_password: "pgpass".to_owned(),
        pg_database: "db".to_owned(),
      }
    );
  }
  #[test]
  fn convert_to_pg_config() {
    env::set_var("PG_HOST", "localhost");
    env::set_var("PG_PORT", "5432");
    env::set_var("PG_USER", "pgadmin");
    env::set_var("PG_PASSWORD", "pgpass");
    env::set_var("PG_DATABASE", "db");
    let config = PGConfig::from_env().unwrap();
    let mut pgc = tokio_postgres::Config::new();
    pgc.host("localhost").port(5432).user("pgadmin").password("pgpass").dbname("db");
    assert_eq!(config.to_pg_config(), pgc);
  }

  #[test]
  fn convert_to_without_db_config() {
    env::set_var("PG_HOST", "localhost");
    env::set_var("PG_PORT", "5432");
    env::set_var("PG_USER", "pgadmin");
    env::set_var("PG_PASSWORD", "pgpass");
    env::set_var("PG_DATABASE", "db");
    let config = PGConfig::from_env().unwrap();
    let mut pgc = tokio_postgres::Config::new();
    pgc.host("localhost").port(5432).user("pgadmin").password("pgpass");
    assert_eq!(config.to_without_db_config(), pgc);
  }

}
