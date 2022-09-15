pub type Result<K, E = DynamoDbErrors> = std::result::Result<K, E>;

#[derive(Debug, thiserror::Error)]
pub enum DynamoDbErrors {
    #[error("Unexpected data from the database")]
    UnexpectedDataFormat(String),

    #[error("DynamoDB request failed")]
    RequestFailure,
}
