use scylla_pg_core::{config::PGConfig, connection::get_client_custom};
use tokio_postgres::Client;

async fn drop_db(client: &Client, database: &str) -> Result<(), tokio_postgres::Error> {
    let drop_db_ddl = format!("DROP DATABASE \"{}\"", database);
    client.execute(&drop_db_ddl, &[]).await?;
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), tokio_postgres::Error> {
    env_logger::builder().format_timestamp_millis().init();

    let mut conf = PGConfig::from_env().unwrap();
    let admin_db = std::env::var("PG_DATABASE_ADMIN").expect("PG_DATABASE_ADMIN is required");
    let to_delete_db = std::env::var("PG_DATABASE").expect("PG_DATABASE is required");
    conf.pg_database = admin_db;
    log::info!("connecting to {} using config user {:?}", conf.pg_database, conf.pg_user);
    let client = get_client_custom(&conf).await?;

    log::info!("Dropping database: {}", &to_delete_db);
    drop_db(&client, &to_delete_db).await?;

    log::info!("Done");
    Ok(())
}
