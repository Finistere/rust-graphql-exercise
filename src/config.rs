use anyhow::Result;
use figment::{
    providers::{Format, Toml},
    Figment,
};
use serde::Deserialize;
use crate::database::dynamodb::DynamoDBConfig;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub dynamodb: DynamoDBConfig,
}

pub fn load() -> Result<Config> {
    let config = Figment::new().merge(Toml::file("App.toml")).extract()?;
    Ok(config)
}
