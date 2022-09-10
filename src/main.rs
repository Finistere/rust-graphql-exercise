use std::future::Future;
use aws_sdk_dynamodb::Client;
use anyhow::Result;
use crate::aws::{DynamoDB, Get};

mod config;
mod graphql;
mod aws;

#[tokio::main]
async fn main() -> Result<()> {
    match run().await {
        Ok(_) => Ok(()),
        Err(e) => {
            println!("{e}");
            Ok(())
        }
    }
}

async fn run() -> Result<()> {
    let config = config::load()?;
    println!("Loaded config");
    let dynamo_db = aws::DynamoDB::load(&config.dynamodb).await?;
    println!("Loaded dynamodb");
    dynamo_db.put("12", 1, "data".as_bytes()).await?;
    println!("Put done");
    let maybe_get = dynamo_db.get("12").await?;
    match maybe_get {
        None => { println!("Unknown key!") }
        Some(get) => { println!("Found {:?}", get) }
    }

    Ok(())
}
