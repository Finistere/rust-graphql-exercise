use anyhow::Result;
use figment::{
    Figment,
    providers::{Format, Toml},
};
use serde::Deserialize;
use tracing::info;

use crate::dynamodb::DynamoDBConfig;
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
