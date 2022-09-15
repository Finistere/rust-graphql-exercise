use async_graphql::{Context, ErrorExtensions, Object, Result};

use crate::dynamodb::DynamoTable;
use crate::graphql::errors::{check_id_kind, Errors};
use crate::graphql::types::ID;

use super::extensions::DynamoTableTodoListExt;
use super::{TodoList, TODO_LIST_TYPE_NAME};

#[derive(Default)]
pub struct TodoListQuery;

#[Object]
impl TodoListQuery {
    async fn todo_list_collection(&self, ctx: &Context<'_>) -> Result<Vec<TodoList>> {
        let dynamodb = ctx.data_unchecked::<DynamoTable>();
        dynamodb.scan_todo_list().await
    }

    pub async fn todo_list(&self, ctx: &Context<'_>, id: ID) -> Result<TodoList> {
        check_id_kind(&id, TODO_LIST_TYPE_NAME)?;
        let dynamodb = ctx.data_unchecked::<DynamoTable>();
        dynamodb
            .get_todo_list(&id)
            .await?
            .ok_or_else(|| Errors::NotFound.extend())
    }
}
