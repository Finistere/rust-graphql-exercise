use async_graphql::{Context, ErrorExtensions, Object, Result};

use crate::dynamodb::DynamoTable;
use crate::graphql::errors::{check_id_kind, Errors};
use crate::graphql::types::ID;

use super::extensions::DynamoTableTodoExt;
use super::{Todo, TODO_TYPE_NAME};

#[derive(Default)]
pub struct TodoQuery;

#[Object]
impl TodoQuery {
    async fn todo_collection(&self, ctx: &Context<'_>) -> Result<Vec<Todo>> {
        let dynamodb = ctx.data_unchecked::<DynamoTable>();
        dynamodb.scan_todo().await
    }

    async fn todo(&self, ctx: &Context<'_>, id: ID) -> Result<Todo> {
        check_id_kind(&id, TODO_TYPE_NAME)?;
        let dynamodb = ctx.data_unchecked::<DynamoTable>();
        dynamodb
            .get_todo(&id)
            .await?
            .ok_or_else(|| Errors::NotFound.extend())
            .map(|(_, todo)| todo)
    }
}
