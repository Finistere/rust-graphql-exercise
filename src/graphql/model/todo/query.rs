use async_graphql::{Context, ErrorExtensions, Object, Result};

use crate::graphql::errors::{check_id_kind, Errors};
use crate::graphql::model::todo::extensions::DynamoTableTodoExt;
use crate::graphql::types::id::ID;
use crate::DynamoTable;

use super::{Todo, TODO_KIND};

#[derive(Default)]
pub struct TodoQuery;

#[Object]
impl TodoQuery {
    async fn todo(&self, ctx: &Context<'_>, id: ID) -> Result<Todo> {
        check_id_kind(&id, TODO_KIND)?;
        let dynamodb = ctx.data_unchecked::<DynamoTable>();
        dynamodb
            .get_todo(&id)
            .await?
            .ok_or(Errors::NotFound.extend())
            .map(|(_, todo)| todo)
    }
}
