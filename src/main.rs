use anyhow::Result;

use crate::database::dynamodb::DynamoDB;
use crate::webserver::run_and_serve;

mod database;
mod config;
mod graphql;
mod webserver;

#[tokio::main]
async fn main() -> Result<()> {
    let config = config::load()?;
    let db = DynamoDB::load(&config.dynamodb).await?;
    run_and_serve(config, db).await?;
    Ok(())
}