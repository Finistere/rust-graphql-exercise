use std::collections::HashMap;
use std::str::FromStr;

use aws_sdk_dynamodb::model::AttributeValue;
use tracing::error;

use super::errors::{DynamoDbErrors, Result};

pub type RawAttributes = HashMap<String, AttributeValue>;

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct ItemKey<K: ToString> {
    pub partition: K,
    pub sort: K,
}

/// Extension used to access easily attributes from the returned HashMap of the AWS SDK.
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
