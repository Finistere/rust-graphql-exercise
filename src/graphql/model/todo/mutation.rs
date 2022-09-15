use async_graphql::{Context, ErrorExtensions, InputObject, Object, OneofObject, Result};

use crate::graphql::errors::{check_id_kind, Errors};
use crate::graphql::model::todo::extensions::DynamoTableTodoExt;
use crate::graphql::types::id::ID;
use crate::DynamoTable;

use super::{Todo, TODO_KIND};

#[derive(Debug, InputObject)]
struct TodoCreateInput {
    title: String,
    complete: Option<bool>,
    list: Option<TodoRelationTodoListLinkInput>,
}

#[derive(Debug, InputObject)]
struct TodoRelationTodoListLinkInput {
    link: ID,
}

#[derive(Debug, InputObject)]
struct TodoUpdateInput {
    id: ID,
    title: Option<String>,
    list: Option<TodoRelationTodoListUpdateInput>,
    complete: Option<bool>,
}

#[derive(Debug, OneofObject)]
enum TodoRelationTodoListUpdateInput {
    Link(ID),
    Unlink(ID),
}

#[derive(Default)]
pub struct TodoMutation;

#[Object]
impl TodoMutation {
    async fn todo_create(&self, ctx: &Context<'_>, input: TodoCreateInput) -> Result<Todo> {
        let todo = Todo {
            id: ID::new(&TODO_KIND),
            title: input.title,
            complete: input.complete.unwrap_or(false),
            list_id: input.list.map(|rel| rel.link),
        };
        let dynamodb = ctx.data_unchecked::<DynamoTable>();
        dynamodb.put_todo(&todo).await.map(|_| todo)
    }

    async fn todo_update(&self, ctx: &Context<'_>, input: TodoUpdateInput) -> Result<Todo> {
        check_id_kind(&input.id, TODO_KIND)?;
        let dynamodb = ctx.data_unchecked::<DynamoTable>();
        let maybe_old = dynamodb.get_todo(&input.id).await?;
        let (old_key, old_todo) = if let Some(x) = maybe_old {
            x
        } else {
            return Err(Errors::NotFound.extend());
        };

        let new_list_id = if let Some(ref relation_update) = input.list {
            match relation_update {
                TodoRelationTodoListUpdateInput::Link(link) => Some(link.clone()),
                TodoRelationTodoListUpdateInput::Unlink(unlink) => {
                    if old_key.partition != *unlink {
                        return Err(Errors::InvalidValue(format!(
                            "Todo '{}' is not linked to the todo list '{}'",
                            input.id, unlink
                        ))
                        .extend());
                    }
                    None
                }
            }
        } else {
            old_todo.list_id.clone()
        };

        let new_todo = Todo {
            id: old_todo.id.clone(),
            title: input.title.unwrap_or(old_todo.title.clone()),
            complete: input.complete.unwrap_or(old_todo.complete.clone()),
            list_id: new_list_id,
        };

        dynamodb
            .update_todo(old_key, old_todo, new_todo)
            .await
            .map(|(_, todo)| todo)
    }

    async fn todo_delete(&self, ctx: &Context<'_>, id: ID) -> Result<Todo> {
        check_id_kind(&id, TODO_KIND)?;
        let dynamodb = ctx.data_unchecked::<DynamoTable>();
        dynamodb
            .delete_todo(&id)
            .await?
            .ok_or(Errors::NotFound.extend())
    }
}
