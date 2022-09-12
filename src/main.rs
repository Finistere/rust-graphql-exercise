use std::convert::Infallible;

use anyhow::Result;
use async_graphql::{EmptyMutation, EmptySubscription, Schema};
use warp::Filter;

use graphql::{GraphQLDatabase, Query};

use crate::database::Database;
use crate::database::dynamodb::DynamoDB;

mod database;
mod config;
mod graphql;

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
    let db: GraphQLDatabase = DynamoDB::load(&config.dynamodb).await?;

    let schema = Schema::build(Query, EmptyMutation, EmptySubscription)
        .data(db)
        .finish();
    println!("{:?}", schema.execute("{ todoCreate(title: \"test\", complete: true) }").await);
    let filter = async_graphql_warp::graphql(schema).and_then(
        |(schema, request): (Schema<Query, EmptyMutation, EmptySubscription>, async_graphql::Request)| async move {
            // Execute query
            let resp = schema.execute(request).await;

            // Return result
            Ok::<_, Infallible>(async_graphql_warp::GraphQLResponse::from(resp))
        },
    );
    warp::serve(filter).run(([0, 0, 0, 0], 8000)).await;
    Ok(())
}
