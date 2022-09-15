use async_graphql::{Error, ErrorExtensions, Result};

use crate::graphql::types::ID;

#[derive(Debug, thiserror::Error)]
pub enum Errors {
    #[error("Could not find resource")]
    NotFound,

    #[error("Invalid value")]
    InvalidValue(String),
}

/// Using the proposed API by async-graphql for the custom errors with additional informations.
impl ErrorExtensions for Errors {
    fn extend(&self) -> Error {
        self.extend_with(|err, e| match err {
            Errors::NotFound => e.set("code", "NOT_FOUND"),
            Errors::InvalidValue(details) => {
                e.set("code", "INVALID_VALUE");
                e.set("details", details.clone());
            }
        })
    }
}

/// Used to ensure that the received ID is for the right type. Otherwise some mutations could
/// alter types there aren't supposed to by passing different ids.
pub fn check_id_kind(id: &ID, expected_kind: &str) -> Result<()> {
    if !id.has_type_name(expected_kind) {
        Err(Errors::InvalidValue(format!("Expected ID with '{}'", expected_kind)).extend())
    } else {
        Ok(())
    }
}
