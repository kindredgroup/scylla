use deadpool_postgres::Pool;
use scylla_pg_core::connection::get_client;
use scylla_pg_core::config;
use dotenv::dotenv;

pub async fn truncate_table() {
    let conf = config::PGConfig::from_env().unwrap();
    let client = get_client(conf.to_pg_config()).await.unwrap();
    let truncate_table_ddl = format!("TRUNCATE task");
    client.execute(&truncate_table_ddl, &[]).await.unwrap();
}