#![allow(dead_code)]
#[path = "../config.rs"]
mod config;
#[path = "../connection.rs"]
mod connection;

use crate::config::PGConfig;
use connection::get_client;
use tokio_postgres::{Client};

#[tokio::main]
async fn main() {
  run_db_operation().await;

}

async fn run_db_operation() {
    dotenv::dotenv().ok();
    let conf = PGConfig::from_env().unwrap();
    let pg_config = conf.to_pg_config();
    match get_client(pg_config).await {
      Ok(client) => {
            let db_result = truncate_task_table(&client).await;
            match db_result {
              Ok(_) => println!("Table task truncated"),
              Err(e) => panic!("{}", e),
            }
      },
      Err(e) =>  panic!("{}", e)
    }
}


async fn truncate_task_table(client: &Client) -> Result<(), tokio_postgres::Error> {
  let truncate_table_ddl = format!("TRUNCATE task");
  client.execute(&truncate_table_ddl, &[]).await?;
  Ok(())
}