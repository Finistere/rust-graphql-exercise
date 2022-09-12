use anyhow::{anyhow, Result};
use async_graphql::*;
use parity_scale_codec::{Decode, Encode};
use ulid::Ulid;

use crate::database::Database;

const VERSION: u32 = 1;

pub type GraphQLDatabase = Box<dyn Database<Id, Vec<u8>>>;

pub struct Query;

#[Object]
impl Query {
    async fn todo_create(&self, ctx: &Context<'_>, title: String, complete: bool) -> Result<Id> {
        let id = Id::new();
        let todo = Todo {
            id,
            title,
            complete,
        };
        ctx.data_unchecked::<GraphQLDatabase>().put(todo.id, VERSION, todo.encode()).await?;
        Ok(id)
    }

    async fn todo(&self, ctx: &Context<'_>, id: Id) -> Result<Todo> {
        if let Some(get) = ctx.data_unchecked::<GraphQLDatabase>().get(id).await? {
            Ok(Todo::decode(&mut get.data.as_slice())?)
        } else {
            Err(anyhow!("Not found"))
        }
    }
}

#[derive(Debug, SimpleObject, Encode, Decode)]
struct Todo {
    id: Id,
    title: String,
    complete: bool,
}

#[derive(Debug, Encode, Decode, Copy, Clone)]
struct Id {
    ulid: u128,
}

impl Id {
    fn new() -> Id {
        Id { ulid: Ulid::new().0 }
    }
}

impl From<Id> for Vec<u8> {
    fn from(id: Id) -> Self {
        id.encode()
    }
}


#[Scalar]
impl ScalarType for Id {
    fn parse(value: Value) -> InputValueResult<Self> {
        if let Value::String(value) = &value {
            Ok(Id {
                ulid: Ulid::from_string(value)?.0,
            })
        } else {
            Err(InputValueError::expected_type(value))
        }
    }

    fn to_value(&self) -> Value {
        Value::String(self.ulid.to_string())
    }
}
