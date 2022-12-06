#![allow(dead_code)]
#[path = "../config.rs"]
mod config;
use tokio_postgres::{error::SqlState, tls::NoTlsStream, Client, Connection, Socket};

mod embedded {
  use refinery::embed_migrations;
  embed_migrations!("src/bin/migrations");
}
#[tokio::main]
async fn main() {
  try_create_db().await.expect("cannot create database");
  run_migrations().await;
}

async fn run_migrations() {
    dotenv::dotenv().ok();
    for item in dotenv::vars() {
      let (key, val) = item;
      println!("{}={}", key, val);
    }
    let conf = config::Config::from_env().unwrap();
    let pg_config = conf.to_pg_config();
    let con = pg_config.connect(tokio_postgres::NoTls).await;
    match con {
      Ok((mut client, connection)) => {
          tokio::spawn(async move {
              if let Err(e) = connection.await {
                eprintln!("connection error: {}", e);
              }
            });
        
            let migration_result = embedded::migrations::runner().run_async(&mut client).await;
            match migration_result {
              Ok(t) => println!("{:?}", t),
              Err(e) => panic!("{}", e),
            }
      },
      Err(e) =>  panic!("{}", e)
    }
}

async fn try_create_db() -> Result<(), tokio_postgres::Error> {
    dotenv::dotenv().ok();
    let conf = config::Config::from_env().unwrap();
    let pg_config = conf.to_without_db_config();
    let (mut client, connection) = pg_config.connect(tokio_postgres::NoTls).await?;
    tokio::spawn(async move {
        if let Err(e) = connection.await {
          eprintln!("connection error: {}", e);
        }
      });

    create_db(&client, conf.pgdatabase.as_str()).await
}

/// Create database
async fn create_db(client: &Client, database: &str) -> Result<(), tokio_postgres::Error> {
    let create_db_ddl = format!("CREATE DATABASE \"{}\" ", database);
    let db_exec_result = client.execute(&create_db_ddl, &[]).await;

    return match db_exec_result {
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