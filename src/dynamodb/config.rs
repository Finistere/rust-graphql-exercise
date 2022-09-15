use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct DynamoDBConfig {
    pub table: String,
    pub partition_key: String,
    pub sort_key: String,
    pub gsi1_name: String,
    pub gsi1_partition_key: String,
    pub gsi1_sort_key: String,
}
