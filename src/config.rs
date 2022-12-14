use anyhow::Result;
use figment::{
    providers::{Format, Toml},
    Figment,
};
use serde::Deserialize;
use tracing::info;

use crate::dynamodb::config::DynamoDBConfig;
use crate::serving::ServingConfig;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub dynamodb: DynamoDBConfig,
    pub serving: ServingConfig,
}

pub fn load() -> Result<Config> {
    let config = Figment::new().merge(Toml::file("App.toml")).extract()?;
    info!("Configuration loaded from App.toml");
    Ok(config)
}
