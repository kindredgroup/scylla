#![allow(dead_code)]
#[path = "../config.rs"]
mod config;
#[path = "../connection.rs"]
mod connection;

use crate::config::PGConfig;
use connection::get_client;

mod embedded {
    use refinery::embed_migrations;
    embed_migrations!("src/bin/migrations");
}
#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();
    log::info!("Migrations");
    let conf = PGConfig::from_env().unwrap();
    log::info!("running migrations for {}", conf.pg_database);
    run_migrations(&conf).await;
}

async fn run_migrations(conf: &PGConfig) {
    let pg_config = conf.to_pg_config();
    match get_client(&pg_config).await {
        Ok(mut client) => {
            log::info!("running migrations {} by {} ", conf.pg_database, conf.pg_user);
            let migration_result = embedded::migrations::runner().run_async(&mut client).await;
            match migration_result {
                Ok(t) => println!("{t:?}"),
                Err(e) => panic!("{e}"),
            }
        }
        Err(e) => {
            log::error!("error connecting to database: {} by user {}", conf.pg_database, conf.pg_user);
            panic!("{}", e)
        }
    }
}
