use figment::{
    providers::{Format, Toml},
    Figment,
};
use serde::Deserialize;
use anyhow::Result;

#[derive(Debug, Deserialize)]
pub struct DynamoDBConfig {
    pub table: String,
}

#[derive(Debug, Deserialize)]
pub struct Config {
    pub dynamodb: DynamoDBConfig,
}

pub fn load() -> Result<Config> {
    let config = Figment::new().merge(Toml::file("App.toml")).extract()?;
    Ok(config)
}
