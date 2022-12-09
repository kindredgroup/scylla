use scylla_pg_lib::manager::DbConfig;
use crate::config::Config;
pub fn get_db_config() -> DbConfig {
    dotenv::dotenv().ok();

    let config = Config::from_env().unwrap();
    DbConfig {
      port: config.pgport,
      db_name: config.pgdatabase,
      password: config.pgpassword,
      user: config.pguser,
      host: config.pghost,
    }
}