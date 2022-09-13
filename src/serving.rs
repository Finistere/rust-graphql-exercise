use actix_web::{App, HttpResponse, HttpServer, web, web::Data};
use async_graphql::http::{GraphQLPlaygroundConfig, playground_source};
use async_graphql_actix_web::{GraphQLRequest, GraphQLResponse};
use serde::Deserialize;
use tracing_actix_web::TracingLogger;

use crate::DynamoDB;
use crate::graphql::build_schema;

use super::graphql::GraphQLSchema;

#[derive(Debug, Deserialize, Clone)]
pub struct ServingConfig {
    pub port: u16,
}

pub async fn run_and_serve(config: ServingConfig, db: DynamoDB) -> () {
    let schema: GraphQLSchema = build_schema(db);

    HttpServer::new(move || {
        App::new()
            .wrap(TracingLogger::default())
            .app_data(Data::new(schema.clone()))
            .configure(configure)
    })
        .bind(("0.0.0.0", config.port))
        .expect("Unable to bind server")
        .run()
        .await
        .expect("Failed to start web server")
}

fn configure(cfg: &mut web::ServiceConfig) {
    cfg
        .service(web::resource("/")
            .route(web::post().to(index))
            .route(web::get().to(index_playground))
        );
}

async fn index(schema: web::Data<GraphQLSchema>, req: GraphQLRequest) -> GraphQLResponse {
    schema.execute(req.into_inner()).await.into()
}

async fn index_playground() -> HttpResponse {
    HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(playground_source(GraphQLPlaygroundConfig::new("/").subscription_endpoint("/")))
}