use std::collections::HashMap;
use std::str::FromStr;

use aws_sdk_dynamodb::client::fluent_builders::{
    DeleteItem, GetItem, PutItem, Query, TransactWriteItems, UpdateItem,
};
use aws_sdk_dynamodb::model::delete::Builder as DeleteBuTler;
use aws_sdk_dynamodb::model::put::Builder as PutBuilder;
use aws_sdk_dynamodb::model::{Delete, Put, TransactWriteItem};
use aws_sdk_dynamodb::output::{
    DeleteItemOutput, GetItemOutput, PutItemOutput, QueryOutput, TransactWriteItemsOutput,
    UpdateItemOutput,
};
use aws_sdk_dynamodb::{model::AttributeValue, Client};
use serde::Deserialize;
use tracing::{error, info};

pub type RawAttributes = HashMap<String, AttributeValue>;
type Result<K, E = DynamoDbErrors> = std::result::Result<K, E>;

#[derive(Debug, Clone, PartialEq)]
pub struct ItemKey<K: ToString> {
    pub partition: K,
    pub sort: K,
}

#[derive(Debug, Deserialize, Clone)]
pub struct DynamoDBConfig {
    table: String,
    pub partition_key: String,
    pub sort_key: String,
    pub gsi1_name: String,
    pub gsi1_partition_key: String,
    pub gsi1_sort_key: String,
}

pub struct DynamoTable {
    client: Client,
    pub config: DynamoDBConfig,
}

impl DynamoTable {
    pub async fn load(config: &DynamoDBConfig) -> anyhow::Result<DynamoTable> {
        let aws_config = aws_config::load_from_env().await;
        let client = Client::new(&aws_config);
        let config = config.clone();

        info!("DynamoDB database configured.");
        Ok(DynamoTable { client, config })
    }

    fn build_key_attributes<K>(&self, key: &ItemKey<K>) -> Option<RawAttributes>
    where
        K: ToString,
    {
        let mut map: RawAttributes = HashMap::new();
        map.insert(
            self.config.partition_key.clone(),
            AttributeValue::S(key.partition.to_string()),
        );
        map.insert(
            self.config.sort_key.clone(),
            AttributeValue::S(key.sort.to_string()),
        );
        Some(map)
    }

    pub fn extract_key<K: FromStr + ToString>(
        &self,
        attributes: &RawAttributes,
    ) -> Result<ItemKey<K>, DynamoDbErrors> {
        Ok(ItemKey {
            partition: attributes.get_from_string(&self.config.partition_key)?,
            sort: attributes.get_from_string(&self.config.sort_key)?,
        })
    }

    pub async fn transact_write<C>(&self, configure: C) -> Result<TransactWriteItemsOutput>
    where
        C: FnOnce(TransactWriteItems) -> TransactWriteItems,
    {
        configure(self.client.transact_write_items())
            .send()
            .await
            .map_err(|e| {
                error!("{}", e);
                DynamoDbErrors::RequestFailure
            })
    }

    pub fn transact_put<K, C>(&self, key: &ItemKey<K>, configure: C) -> TransactWriteItem
    where
        K: ToString,
        C: FnOnce(PutBuilder) -> PutBuilder,
    {
        TransactWriteItem::builder()
            .put(
                configure(
                    Put::builder()
                        .table_name(&self.config.table)
                        .item(
                            &self.config.partition_key,
                            AttributeValue::S(key.partition.to_string()),
                        )
                        .item(
                            &self.config.sort_key,
                            AttributeValue::S(key.sort.to_string()),
                        ),
                )
                .build(),
            )
            .build()
    }

    pub fn transact_delete<K, C>(&self, key: &ItemKey<K>, configure: C) -> TransactWriteItem
    where
        K: ToString,
        C: FnOnce(DeleteBuTler) -> DeleteBuTler,
    {
        TransactWriteItem::builder()
            .delete(
                configure(
                    Delete::builder()
                        .table_name(&self.config.table)
                        .set_key(self.build_key_attributes(key)),
                )
                .build(),
            )
            .build()
    }

    pub async fn query_gsi1_get<K, C>(
        &self,
        gsi1_key: &ItemKey<K>,
        configure: C,
    ) -> Result<QueryOutput>
    where
        K: ToString,
        C: FnOnce(Query) -> Query,
    {
        let req = self
            .client
            .query()
            .table_name(&self.config.table)
            .index_name(&self.config.gsi1_name)
            .key_condition_expression("#pk = :pk AND #sk = :sk")
            .expression_attribute_names("#pk", &self.config.gsi1_partition_key)
            .expression_attribute_names("#sk", &self.config.gsi1_sort_key)
            .expression_attribute_values(":pk", AttributeValue::S(gsi1_key.partition.to_string()))
            .expression_attribute_values(":sk", AttributeValue::S(gsi1_key.sort.to_string()));
        configure(req).send().await.map_err(|e| {
            error!("{}", e);
            DynamoDbErrors::RequestFailure
        })
    }

    pub fn query_partition_by_prefix<K: ToString>(&self, pkey: K, prefix: &String) -> Query {
        self.client
            .query()
            .table_name(&self.config.table)
            .key_condition_expression("#pk = :pk and begins_with(#sk, :sk)")
            .expression_attribute_names("#pk", &self.config.partition_key)
            .expression_attribute_names("#sk", &self.config.sort_key)
            .expression_attribute_values(":pk", AttributeValue::S(pkey.to_string()))
            .expression_attribute_values(":sk", AttributeValue::S(prefix.clone()))
    }

    pub async fn get_item<K, C>(&self, key: &ItemKey<K>, configure: C) -> Result<GetItemOutput>
    where
        K: ToString,
        C: FnOnce(GetItem) -> GetItem,
    {
        let req = self
            .client
            .get_item()
            .table_name(&self.config.table)
            .set_key(self.build_key_attributes(key));
        configure(req).send().await.map_err(|e| {
            error!("{}", e);
            DynamoDbErrors::RequestFailure
        })
    }

    pub async fn put_item<K, C>(&self, key: &ItemKey<K>, configure: C) -> Result<PutItemOutput>
    where
        K: ToString + Clone,
        C: FnOnce(PutItem) -> PutItem,
    {
        let req = self
            .client
            .put_item()
            .table_name(&self.config.table)
            .item(
                &self.config.partition_key,
                AttributeValue::S(key.partition.to_string()),
            )
            .item(
                &self.config.sort_key,
                AttributeValue::S(key.sort.to_string()),
            );
        configure(req).send().await.map_err(|e| {
            error!("{}", e);
            DynamoDbErrors::RequestFailure
        })
    }

    pub async fn delete_item<K, C>(
        &self,
        key: &ItemKey<K>,
        configure: C,
    ) -> Result<DeleteItemOutput>
    where
        K: ToString,
        C: FnOnce(DeleteItem) -> DeleteItem,
    {
        let req = self
            .client
            .delete_item()
            .table_name(&self.config.table)
            .set_key(self.build_key_attributes(key));
        configure(req).send().await.map_err(|e| {
            error!("{}", e);
            DynamoDbErrors::RequestFailure
        })
    }

    pub async fn update_item<K, C>(
        &self,
        key: &ItemKey<K>,
        configure: C,
    ) -> Result<UpdateItemOutput>
    where
        K: ToString,
        C: FnOnce(UpdateItem) -> UpdateItem,
    {
        let req = self
            .client
            .update_item()
            .table_name(&self.config.table)
            .set_key(self.build_key_attributes(key));
        configure(req).send().await.map_err(|e| {
            error!("{}", e);
            DynamoDbErrors::RequestFailure
        })
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
    fn get_from_string<F: FromStr>(&self, key: &str) -> Result<F>;
    fn get_string(&self, key: &str) -> Result<&String>;
    fn get_bool(&self, key: &str) -> Result<&bool>;
}

impl AttributesGetterExt for RawAttributes {
    fn get_from_string<F: FromStr>(&self, key: &str) -> Result<F> {
        let attr_n = get_attr(self, key)?.as_s().map_err(|_| {
            let message = format!("Expected key '{}' to be a string", key);
            error!(message);
            DynamoDbErrors::UnexpectedDataFormat(message)
        })?;
        let parsed: F = attr_n.parse().map_err(|_e| {
            let message = format!("Could not parse '{}'", attr_n);
            error!(message);
            DynamoDbErrors::UnexpectedDataFormat(message)
        })?;
        Ok(parsed)
    }

    fn get_string(&self, key: &str) -> Result<&String> {
        get_attr(self, key)?.as_s().map_err(|_| {
            let message = format!("Expected key '{}' to be a string", key);
            error!(message);
            DynamoDbErrors::UnexpectedDataFormat(message)
        })
    }

    fn get_bool(&self, key: &str) -> Result<&bool> {
        get_attr(self, key)?.as_bool().map_err(|_| {
            let message = format!("Expected key '{}' to be a bool", key);
            error!(message);
            DynamoDbErrors::UnexpectedDataFormat(message)
        })
    }
}

fn get_attr<'a>(map: &'a RawAttributes, key: &str) -> Result<&'a AttributeValue> {
    if let Some(value) = map.get(key) {
        Ok(value)
    } else {
        let message = format!("Missing key '{}'", key);
        error!(message);
        Err(DynamoDbErrors::UnexpectedDataFormat(message))
    }
}
