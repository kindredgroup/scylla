use scylla_pg_core::{
    config::PGConfig,
    connection::{get_client, get_client_custom},
};
use tokio_postgres::{error::SqlState, Client};

#[tokio::main]
async fn main() -> Result<(), tokio_postgres::Error> {
    log::info!("creating database");
    env_logger::builder().format_timestamp_millis().init();
    let template = std::env::var("PG_DATABASE_TEMPLATE");

    let template_name = match template {
        Ok(t) => Some(t),
        Err(_) => {
            log::info!("$PG_DATABASE_TEMPLATE not set, creating database without template.");
            None
        }
    };
    let admin_db = std::env::var("PG_DATABASE_ADMIN").expect("PG_DATABASE_ADMIN is required");
    let db_to_create = std::env::var("PG_DATABASE").expect("PG_DATABASE is required");
    let mut conf = PGConfig::from_env().unwrap();
    conf.pg_database = admin_db;
    let admin_db_client = get_client_custom(&conf).await?;

    create_db(&admin_db_client, &db_to_create, &template_name).await?;

    log::info!("{} database created successfully", &db_to_create);

    log::info!("connect to new database");
    let conf = PGConfig::from_env().unwrap();
    let to_db_config = conf.to_pg_config();
    let new_db_client = get_client(&to_db_config).await?;

    if template_name.is_some() {
        let mig_user = std::env::var("PG_MIG_USER").expect("PG_MIG_USER is required");
        log::info!("assigning permissions to database {} for user {} ", conf.pg_database, mig_user);
        assign_permissions(&new_db_client, &conf.pg_database, &mig_user).await
    } else {
        log::info!("no permissions to assign");
        Ok(())
    }
}

async fn create_db(client: &Client, database_name: &str, db_template: &Option<String>) -> Result<(), tokio_postgres::Error> {
    let create_db_ddl = if let Some(db_template_name) = db_template {
        log::info!("creating database {} with template: {}", database_name, db_template_name);
        format!("CREATE DATABASE \"{}\" TEMPLATE \"{}\"", database_name, db_template_name)
    } else {
        format!("CREATE DATABASE \"{}\"", database_name)
    };
    let db_exec_result = client.execute(&create_db_ddl, &[]).await;
    match db_exec_result {
        Ok(_) => Ok(()),
        Err(e) => {
            if e.code() == Some(&SqlState::DUPLICATE_DATABASE) {
                eprintln!("database already exists. proceeding with migrations.");
                Ok(())
            } else {
                Err(e)
            }
        }
    }
}

async fn assign_permissions(client: &Client, database_name: &str, user: &str) -> Result<(), tokio_postgres::Error> {
    let grant = format!("GRANT CONNECT ON DATABASE \"{}\" TO \"{}\"", database_name, user);
    client.execute(&grant, &[]).await?;

    let grant = format!("GRANT USAGE ON SCHEMA public TO \"{}\"", user);
    client.execute(&grant, &[]).await?;

    let grant = format!(
        "ALTER DEFAULT PRIVILEGES IN SCHEMA public GRANT SELECT, INSERT, UPDATE, DELETE ON TABLES TO \"{}\"",
        user
    );
    client.execute(&grant, &[]).await?;
    Ok(())
}
