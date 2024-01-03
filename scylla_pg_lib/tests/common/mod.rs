use scylla_pg_core::config;
use scylla_pg_core::connection::get_client;
use scylla_pg_lib::manager::PgManager;

pub async fn truncate_table() {
    let conf = config::PGConfig::from_env().unwrap();
    let client = get_client(&(conf.to_pg_config())).await.unwrap();
    let truncate_table_ddl = "TRUNCATE task".to_string();
    client.execute(&truncate_table_ddl, &[]).await.unwrap();
}

pub async fn get_pg_manager() -> PgManager {
    let config = config::PGConfig::from_env().unwrap();
    PgManager::from_config(&config).expect("Error creating PgManager Instance")
}
