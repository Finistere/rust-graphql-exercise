use anyhow::{anyhow, Result};
use aws_sdk_dynamodb::{Client, model::AttributeValue, types::Blob};
use thiserror::Error;

use super::config::DynamoDBConfig;

pub struct DynamoDB<'a> {
    client: Client,
    table: &'a String,
}

#[derive(Debug)]
pub struct Get {
    data: Vec<u8>,
    version: u32,
}

#[derive(Error, Debug)]
pub enum DatabaseErrors {
    #[error("Invalid or missing version for {id:?}")]
    InvalidVersionError { id: String },
    #[error("Invalid or missing data for {id:?}")]
    InvalidDataError { id: String },
}

impl<'a> DynamoDB<'a> {
    pub async fn load(config: &DynamoDBConfig) -> Result<DynamoDB> {
        let aws_config = aws_config::load_from_env().await;
        let client = Client::new(&aws_config);
        Ok(DynamoDB {
            client,
            table: &config.table,
        })
    }

    pub async fn get(&self, id: &str) -> Result<Option<Get>> {
        let item = self
            .client
            .get_item()
            .table_name(self.table)
            .key("id", AttributeValue::S(String::from(id)))
            .send()
            .await?;
        return if let Some(map) = item.item() {
            if let (Some(attr), Some(version)) = (map.get("data"), map.get("version")) {
                let data = attr.as_b().map_err(|_| anyhow!("Invalid data for {:?}", id))?.to_owned().into_inner();
                let version: u32 = version.as_n().map_err(|_| anyhow!("Invalid version for {:?}", id))?.parse()?;
                Ok(Some(Get { version, data }))
            } else {
                Err(anyhow!("Missing version or data for {:?}", id))
            }
        } else {
            Ok(None)
        }
    }

    pub async fn put(&self, id: &str, version: u32, data: &[u8]) -> Result<()> {
        self.client
            .put_item()
            .table_name(self.table)
            .item("id", AttributeValue::S(String::from(id)))
            .item("version", AttributeValue::N(version.to_string()))
            .item("data", AttributeValue::B(Blob::new(base64::encode(data))))
            .send()
            .await?;
        Ok(())
    }
}
