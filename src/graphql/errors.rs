use async_graphql::{
    Error, ErrorExtensions,
};

#[derive(Debug, thiserror::Error)]
pub enum Errors {
    #[error("Could not find resource")]
    NotFound,

    #[error("Invalid value")]
    InvalidValue(String),
}

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
