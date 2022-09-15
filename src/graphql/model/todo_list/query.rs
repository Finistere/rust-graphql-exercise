use async_graphql::{Context, ErrorExtensions, Object, Result};

use crate::graphql::errors::{check_id_kind, Errors};
use crate::graphql::types::id::ID;
use crate::DynamoTable;

use super::extensions::DynamoTableTodoListExt;
use super::{TodoList, TODO_LIST_KIND};

#[derive(Default)]
pub struct TodoListQuery;

#[Object]
impl TodoListQuery {
    pub async fn todo_list(&self, ctx: &Context<'_>, id: ID) -> Result<TodoList> {
        check_id_kind(&id, TODO_LIST_KIND)?;
        let dynamodb = ctx.data_unchecked::<DynamoTable>();
        dynamodb
            .get_todo_list(&id)
            .await?
            .ok_or(Errors::NotFound.extend())
    }
}
