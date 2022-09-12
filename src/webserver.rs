use actix_web::{App, guard, HttpServer, web, web::Data};
use async_graphql::{EmptyMutation, EmptySubscription, Schema};
use async_graphql_actix_web::{GraphQLRequest, GraphQLResponse};

use super::config::Config;
use super::graphql::{GraphQLDatabase, GraphQLSchema, Query};

pub async fn run_and_serve(_config: Config, db: GraphQLDatabase) -> std::io::Result<()> {
    let schema: GraphQLSchema = Schema::build(Query, EmptyMutation, EmptySubscription)
        .data(db)
        .finish();

    HttpServer::new(move || {
        App::new()
            .app_data(Data::new(schema.clone()))
            .service(web::resource("/").guard(guard::Post()).to(index))
    })
        .bind(("127.0.0.1", 8000))?
        .run()
        .await
}


async fn index(schema: web::Data<GraphQLSchema>, req: GraphQLRequest) -> GraphQLResponse {
    schema.execute(req.into_inner()).await.into()
}