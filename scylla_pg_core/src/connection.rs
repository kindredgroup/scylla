use crate::config::PGConfig;
use deadpool_postgres::{Manager, ManagerConfig, Pool, RecyclingMethod};
use tokio_postgres::{NoTls, Client, Connection, Config, tls::NoTlsStream};

pub fn get_pool(config: PGConfig) -> Result<Pool, deadpool_postgres::BuildError> {
    let mgr_config = ManagerConfig {
      recycling_method: RecyclingMethod::Fast,
    };
    let mgr = Manager::from_config(config.to_pg_config(), NoTls, mgr_config);
   Pool::builder(mgr).max_size(16).build()
}

pub async fn get_client(config: Config) -> Result<Client, tokio_postgres::Error> {
    let (client, connection) = config.connect(tokio_postgres::NoTls).await?;
    tokio::spawn(async move {
        if let Err(e) = connection.await {
             eprintln!("connection error: {}", e);
        }
    });
    Ok(client)
}