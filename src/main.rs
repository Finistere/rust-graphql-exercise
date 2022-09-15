use tracing_bunyan_formatter::{BunyanFormattingLayer, JsonStorageLayer};
use tracing_log::LogTracer;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::{EnvFilter, Registry};

use dynamodb::DynamoTable;

use crate::serving::run_and_serve;

mod config;
mod dynamodb;
mod graphql;
mod serving;

#[tokio::main]
async fn main() -> () {
    // Tracing
    LogTracer::init().expect("Unable to setup log tracer!");
    let app_name = concat!(env!("CARGO_PKG_NAME"), "-", env!("CARGO_PKG_VERSION")).to_string();
    let (non_blocking, _guard) = tracing_appender::non_blocking(std::io::stdout());
    let bunyan_formatting_layer = BunyanFormattingLayer::new(app_name, non_blocking);
    let subscriber = Registry::default()
        .with(EnvFilter::new(
            option_env!("TRACING_LEVEL").unwrap_or("INFO"),
        ))
        .with(JsonStorageLayer)
        .with(bunyan_formatting_layer);
    tracing::subscriber::set_global_default(subscriber).unwrap();

    let config = config::load().unwrap();
    let db = DynamoTable::load(&config.dynamodb).await.unwrap();

    run_and_serve(config.serving, db).await;
}
