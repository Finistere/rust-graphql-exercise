use std::collections::HashMap;
use std::str::FromStr;

use aws_sdk_dynamodb::{Client, model::AttributeValue};
use aws_sdk_dynamodb::client::fluent_builders::{DeleteItem, GetItem, PutItem, Query, TransactWriteItems, UpdateItem};
use aws_sdk_dynamodb::model::{Delete, Put, TransactWriteItem};
use aws_sdk_dynamodb::model::delete::Builder as DeleteBuidler;
use aws_sdk_dynamodb::model::put::Builder as PutBuilder;
use aws_sdk_dynamodb::output::{DeleteItemOutput, GetItemOutput, PutItemOutput, QueryOutput, TransactWriteItemsOutput, UpdateItemOutput};
use serde::Deserialize;
use tracing::{error, info};

use crate::graphql::id::{ID, Kind};

pub type Item = HashMap<String, AttributeValue>;


#[derive(Debug, Deserialize, Clone)]
pub struct DynamoDBConfig {
    table: String,
    pub partition_key: String,
    pub sort_key: String,
    pub gsi1_name: String,
    pub gsi1_partition_key: String,
    pub gsi1_sort_key: String,
}

pub struct DynamoDB {
    client: Client,
    pub config: DynamoDBConfig,
}

impl DynamoDB {
    pub async fn load(config: &DynamoDBConfig) -> anyhow::Result<DynamoDB> {
        let aws_config = aws_config::load_from_env().await;
        let client = Client::new(&aws_config);
        let config = config.clone();

        info!("DynamoDB database configured.");
        Ok(DynamoDB { client, config })
    }

    pub async fn transact_write<F: FnOnce(TransactWriteItems) -> TransactWriteItems>(&self, build: F) -> Result<TransactWriteItemsOutput, DynamoDbErrors> {
        build(self.client.transact_write_items())
            .send()
            .await
            .map_err(|e| {
                error!("{}", e);
                DynamoDbErrors::RequestFailure
            })
    }

    pub fn transact_put<F: FnOnce(PutBuilder) -> PutBuilder>(&self, pkey: &ID, skey: &ID, build: F) -> TransactWriteItem {
        TransactWriteItem::builder()
            .put(build(Put::builder().table_name(&self.config.table)
                .item(&self.config.partition_key, AttributeValue::S(String::from(pkey)))
                .item(&self.config.sort_key, AttributeValue::S(String::from(skey)))).build())
            .build()
    }

    pub fn transact_delete<F: FnOnce(DeleteBuidler) -> DeleteBuidler>(&self, pkey: &ID, skey: &ID, build: F) -> TransactWriteItem {
        TransactWriteItem::builder()
            .delete(build(Delete::builder()
                .table_name(&self.config.table)
                .set_key(self.build_key(pkey, skey))).build())
            .build()
    }

    pub async fn query_gsi1_get<F: FnOnce(Query) -> Query>(&self, gsi1_pkey: &ID, gsi1_skey: &ID, build: F) -> Result<QueryOutput, DynamoDbErrors> {
        let req = self.client
            .query()
            .table_name(&self.config.table)
            .index_name(&self.config.gsi1_name)
            .key_condition_expression("#pk = :pk AND #sk = :sk")
            .expression_attribute_names("#pk", &self.config.gsi1_partition_key)
            .expression_attribute_names("#sk", &self.config.gsi1_sort_key)
            .expression_attribute_values(":pk", AttributeValue::S(String::from(gsi1_pkey)))
            .expression_attribute_values(":sk", AttributeValue::S(String::from(gsi1_skey)));
        build(req)
            .send()
            .await
            .map_err(|e| {
                error!("{}", e);
                DynamoDbErrors::RequestFailure
            })
    }

    pub async fn query_by_kind<F: FnOnce(Query) -> Query>(&self, pkey: &ID, kind: &Kind, build: F) -> Result<QueryOutput, DynamoDbErrors> {
        let req = self.client
            .query()
            .table_name(&self.config.table)
            .key_condition_expression("#pk = :pk and begins_with(#sk, :sk)")
            .expression_attribute_names("#pk", &self.config.partition_key)
            .expression_attribute_names("#sk", &self.config.sort_key)
            .expression_attribute_values(":pk", AttributeValue::S(String::from(pkey)))
            .expression_attribute_values(":sk", AttributeValue::S(ID::prefix(kind)));
        build(req)
            .send()
            .await
            .map_err(|e| {
                error!("{}", e);
                DynamoDbErrors::RequestFailure
            })
    }

    pub async fn get_item<F: FnOnce(GetItem) -> GetItem>(&self, pkey: &ID, skey: &ID, build: F) -> Result<GetItemOutput, DynamoDbErrors> {
        let req = self.client
            .get_item()
            .table_name(&self.config.table)
            .set_key(self.build_key(pkey, skey));
        build(req)
            .send()
            .await
            .map_err(|e| {
                error!("{}", e);
                DynamoDbErrors::RequestFailure
            })
    }

    pub async fn put_item<F: FnOnce(PutItem) -> PutItem>(&self, pkey: &ID, skey: &ID, build: F) -> Result<PutItemOutput, DynamoDbErrors> {
        let req = self.client
            .put_item()
            .table_name(&self.config.table)
            .item(&self.config.partition_key, AttributeValue::S(String::from(pkey)))
            .item(&self.config.sort_key, AttributeValue::S(String::from(skey)));
        build(req).send().await.map_err(|e| {
            error!("{}", e);
            DynamoDbErrors::RequestFailure
        })
    }

    pub async fn delete_item<F: FnOnce(DeleteItem) -> DeleteItem>(&self, pkey: &ID, skey: &ID, build: F) -> Result<DeleteItemOutput, DynamoDbErrors> {
        let req = self.client
            .delete_item()
            .table_name(&self.config.table)
            .set_key(self.build_key(pkey, skey));
        build(req)
            .send()
            .await
            .map_err(|e| {
                error!("{}", e);
                DynamoDbErrors::RequestFailure
            })
    }

    pub async fn update_item<F: FnOnce(UpdateItem) -> UpdateItem>(&self, pkey: &ID, skey: &ID, build: F) -> Result<UpdateItemOutput, DynamoDbErrors> {
        let req = self.client
            .update_item()
            .table_name(&self.config.table)
            .set_key(self.build_key(pkey, skey));
        build(req).send().await.map_err(|e| {
            error!("{}", e);
            DynamoDbErrors::RequestFailure
        })
    }

    pub fn build_key(&self, pkey: &ID, skey: &ID) -> Option<Item> {
        let mut map: Item = HashMap::new();
        map.insert(self.config.partition_key.clone(), AttributeValue::S(String::from(pkey)));
        map.insert(self.config.sort_key.clone(), AttributeValue::S(String::from(skey)));
        Some(map)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum DynamoDbErrors {
    #[error("Unexpected data from the database")]
    UnexpectedDataFormat(String),

    #[error("DynamoDB request failed")]
    RequestFailure,
}

pub trait AttributesGetterExt {
    fn get_id(&self, key: &str) -> Result<ID, DynamoDbErrors>;
    fn get_number<F: FromStr>(&self, key: &str) -> Result<F, DynamoDbErrors>;
    fn get_string(&self, key: &str) -> Result<&String, DynamoDbErrors>;
    fn get_bool(&self, key: &str) -> Result<&bool, DynamoDbErrors>;
}

impl AttributesGetterExt for Item {
    fn get_id(&self, key: &str) -> Result<ID, DynamoDbErrors> {
        let value = self.get_string(key)?;
        ID::from_string(value).map_err(|_e| {
            let message = format!("Could not parse ID from '{}'", value);
            error!(message);
            DynamoDbErrors::UnexpectedDataFormat(message)
        })
    }

    fn get_number<F: FromStr>(&self, key: &str) -> Result<F, DynamoDbErrors> {
        let attr_n = get_attr(self, key)?.as_n().map_err(|_| {
            let message = format!("Expected key '{}' to be a number", key);
            error!(message);
            DynamoDbErrors::UnexpectedDataFormat(message)
        })?;
        let number: F = attr_n.parse().map_err(|_e| {
            let message = format!("Could not parse '{}'", attr_n);
            error!(message);
            DynamoDbErrors::UnexpectedDataFormat(message)
        })?;
        Ok(number)
    }

    fn get_string(&self, key: &str) -> Result<&String, DynamoDbErrors> {
        get_attr(self, key)?.as_s().map_err(|_| {
            let message = format!("Expected key '{}' to be a number", key);
            error!(message);
            DynamoDbErrors::UnexpectedDataFormat(message)
        })
    }

    fn get_bool(&self, key: &str) -> Result<&bool, DynamoDbErrors> {
        get_attr(self, key)?.as_bool().map_err(|_| {
            let message = format!("Expected key '{}' to be a number", key);
            error!(message);
            DynamoDbErrors::UnexpectedDataFormat(message)
        })
    }
}


fn get_attr<'a>(map: &'a Item, key: &str) -> Result<&'a AttributeValue, DynamoDbErrors> {
    if let Some(value) = map.get(key) {
        Ok(value)
    } else {
        let message = format!(
            "Missing key '{}'",
            key
        );
        error!(message);
        Err(DynamoDbErrors::UnexpectedDataFormat(message))
    }
}
