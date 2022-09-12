use std::fmt::Display;

use anyhow::{anyhow, Result};
use async_trait::async_trait;
use aws_sdk_dynamodb::{Client, model::AttributeValue, types::Blob};
use serde::Deserialize;

use super::{Database, Get};

#[derive(Debug, Deserialize, Clone)]
pub struct DynamoDBConfig {
    pub table: String,
}

pub struct DynamoDB {
    client: Client,
    config: DynamoDBConfig,
}

#[async_trait]
impl<Id, Data> Database<Id, Data> for DynamoDB
    where Id: Into<Vec<u8>> + Send + Display + Copy,
          Data: Into<Vec<u8>> + Send {
    async fn put(&self, id: Id, version: u32, data: Data) -> Result<()>
        where Id: 'async_trait,
              Data: 'async_trait {
        self.client
            .put_item()
            .table_name(&self.config.table)
            .item("id", DynamoDB::attr_b(id))
            .item("version", AttributeValue::N(version.to_string()))
            .item("data", DynamoDB::attr_b(data))
            .send()
            .await?;
        Ok(())
    }

    async fn get(&self, id: Id) -> Result<Option<Get>>
        where Id: 'async_trait {
        let item = self
            .client
            .get_item()
            .table_name(&self.config.table)
            .key("id", DynamoDB::attr_b(id))
            .send()
            .await?;
        if let Some(map) = item.item() {
            if let (Some(attr), Some(version)) = (map.get("data"), map.get("version")) {
                let blob = attr
                    .as_b()
                    .map_err(|_| anyhow!("Invalid data for {}", id))?
                    .to_owned();

                let version: u32 = version
                    .as_n()
                    .map_err(|_| anyhow!("Invalid version for {}", id))?
                    .parse()?;
                Ok(Some(Get { version, data: blob.into_inner() }))
            } else {
                Err(anyhow!("Missing version or data for {}", id))
            }
        } else {
            Ok(None)
        }
    }
}


impl DynamoDB {
    pub async fn load<Id, Data>(config: &DynamoDBConfig) -> Result<Box<dyn Database<Id, Data>>>
        where Id: Into<Vec<u8>> + Send + Display + Copy,
              Data: Into<Vec<u8>> + Send {
        let aws_config = aws_config::load_from_env().await;
        let client = Client::new(&aws_config);
        let config = config.clone();
        Ok(Box::new(DynamoDB { client, config }))
    }

    fn attr_b<T: Into<Vec<u8>>>(bytes: T) -> AttributeValue {
        AttributeValue::B(Blob::new(bytes))
    }
}
