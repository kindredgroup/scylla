#![allow(dead_code)]
#[path = "../config.rs"]
mod config;
#[path = "../connection.rs"]
mod connection;

use crate::config::PGConfig;
use connection::get_client;
use tokio_postgres::{error::SqlState, Client};

mod embedded {
    use refinery::embed_migrations;
    embed_migrations!("src/bin/migrations");
}
#[tokio::main]
async fn main() {
    try_create_db().await.expect("could not create database");
    run_migrations().await;
}

async fn run_migrations() {
    dotenv::dotenv().ok();
    let conf = PGConfig::from_env().unwrap();
    let pg_config = conf.to_pg_config();
    match get_client(&pg_config).await {
        Ok(mut client) => {
            let migration_result = embedded::migrations::runner().run_async(&mut client).await;
            match migration_result {
                Ok(t) => println!("{:?}", t),
                Err(e) => panic!("{}", e),
            }
        }
        Err(e) => panic!("{}", e),
    }
}

async fn try_create_db() -> Result<(), tokio_postgres::Error> {
    dotenv::dotenv().ok();
    let conf = PGConfig::from_env().unwrap();
    let pg_config = conf.to_without_db_config();
    let client = get_client(&pg_config).await?;
    create_db(&client, conf.pg_database.as_str()).await
}

/// Create database
async fn create_db(client: &Client, database: &str) -> Result<(), tokio_postgres::Error> {
    let create_db_ddl = format!("CREATE DATABASE \"{}\" ", database);
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
