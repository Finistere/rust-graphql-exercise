use async_graphql::{Context, ErrorExtensions, InputObject, Object, Result};

use crate::dynamodb::DynamoTable;
use crate::graphql::errors::{check_id_kind, Errors};
use crate::graphql::types::ID;

use super::extensions::DynamoTableTodoListExt;
use super::{TodoList, TODO_LIST_TYPE_NAME};

#[derive(Debug, InputObject)]
struct TodoListInputCreate {
    title: String,
}

#[derive(Debug, InputObject)]
struct TodoListInputUpdate {
    id: ID,
    title: Option<String>,
}

#[derive(Default)]
pub struct TodoListMutation;

#[Object]
impl TodoListMutation {
    async fn todo_list_create(
        &self,
        ctx: &Context<'_>,
        input: TodoListInputCreate,
    ) -> Result<TodoList> {
        let dynamodb = ctx.data_unchecked::<DynamoTable>();
        let todo_list = TodoList {
            id: ID::new(TODO_LIST_TYPE_NAME),
            title: input.title,
        };
        dynamodb.put_todo_list(&todo_list).await?;
        Ok(todo_list)
    }

    async fn todo_list_update(
        &self,
        ctx: &Context<'_>,
        input: TodoListInputUpdate,
    ) -> Result<TodoList> {
        check_id_kind(&input.id, TODO_LIST_TYPE_NAME)?;
        let dynamodb = ctx.data_unchecked::<DynamoTable>();

        if let Some(title) = input.title {
            dynamodb.update_todo_list(&input.id, title).await
        } else {
            dynamodb
                .get_todo_list(&input.id)
                .await?
                .ok_or_else(|| Errors::NotFound.extend())
        }
    }

    async fn todo_list_delete(&self, ctx: &Context<'_>, id: ID) -> Result<TodoList> {
        check_id_kind(&id, TODO_LIST_TYPE_NAME)?;
        let dynamodb = ctx.data_unchecked::<DynamoTable>();
        dynamodb
            .delete_todo_list(&id)
            .await?
            .ok_or_else(|| Errors::NotFound.extend())
    }
}
