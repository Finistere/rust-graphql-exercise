use std::convert::Into;

use anyhow::Result;
use async_trait::async_trait;

pub mod dynamodb;


#[derive(Debug)]
pub struct Get {
    pub version: u32,
    pub data: Vec<u8>,
}

// async_trait adds the Send trait to the future which wouldn't be
// necessary in a CloudFlare worker/lambda where everything is executed on a single core
// So it would be worth defining the impl Future by hand.
#[async_trait]
pub trait Database<Id, Data>: Send + Sync
    where Id: Into<Vec<u8>> + Send,
          Data: Into<Vec<u8>> + Send {
    async fn get(&self, id: Id) -> Result<Option<Get>>
        where Id: 'async_trait;

    async fn put(&self, id: Id, version: u32, data: Data) -> Result<()>
        where Id: 'async_trait,
              Data: 'async_trait;
}