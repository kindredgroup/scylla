// $coverage:ignore-start
//! Ignored from coverage because of real database interactions. covered as part of component tests
use crate::config::PGConfig;
use deadpool_postgres::{Manager, ManagerConfig, Pool, RecyclingMethod};
use tokio_postgres::{Client, Config, NoTls};

/// This function returns an instance of pool created by `deadpool_postgres`.
/// # Arguments
/// Take reference of `PGConfig`
/// # Example
/// let pool = `get_pool(&config)?;`
/// # Errors
/// This could return `deadpool_postgres::BuildError`
pub fn get_pool(config: &PGConfig) -> Result<Pool, deadpool_postgres::BuildError> {
    let mgr_config = ManagerConfig {
        recycling_method: RecyclingMethod::Fast,
    };
    let mgr = Manager::from_config(config.to_pg_config(), NoTls, mgr_config);
    Pool::builder(mgr).max_size(16).build()
}
/// This function returns an instance of `PostgresSQL Client` created by `tokio_postgres`.
/// # Arguments
/// Take instance of `tokio_postgres::config`
/// # Example
/// let client = `get_client(config)?;`
/// # Errors
/// This could return `tokio_postgres::Error`
pub async fn get_client(config: &Config) -> Result<Client, tokio_postgres::Error> {
    let (client, connection) = config.connect(tokio_postgres::NoTls).await?;
    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("connection error: {}", e);
        }
    });
    Ok(client)
}
