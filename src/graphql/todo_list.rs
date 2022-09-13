use std::convert::identity;

use async_graphql::{Context, ErrorExtensions, InputObject, Object, Result, SimpleObject};
use aws_sdk_dynamodb::model::AttributeValue;

use crate::DynamoDB;
use crate::dynamodb::AttributesGetterExt;
use crate::graphql::errors::Errors;
use crate::graphql::id::{ID, Kind};
use crate::graphql::todo::{Todo, TODO_KIND};

const TODO_LIST_KIND: &str = "todo_list";

#[derive(Debug, SimpleObject)]
pub struct TodoList {
    id: ID,
    title: String,
    todos: Vec<Todo>,
}

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
pub struct TodoListQuery;

#[Object]
impl TodoListQuery {
    pub async fn todo_list(&self, ctx: &Context<'_>, id: ID) -> Result<TodoList> {
        let dynamodb = ctx.data_unchecked::<DynamoDB>();
        let output = dynamodb.get_item(&id, &id, identity).await?;
        if let Some(item) = output.item() {
            let mut todos: Vec<Todo> = Vec::new();
            if ctx.look_ahead().field("todos").exists() {
                let todo_kind = Kind::from_string(TODO_KIND);
                let out = dynamodb.query_by_kind(&id, &todo_kind, identity).await?;
                for attributes in out.items().unwrap_or_default() {
                    todos.push(Todo::try_from_attributes(attributes.get_id(&dynamodb.config.sort_key)?, Some(attributes))?);
                }
            }
            Ok(TodoList {
                id,
                title: item.get_string("title")?.clone(),
                todos,
            })
        } else {
            Err(Errors::NotFound.extend())
        }
    }
}

#[derive(Default)]
pub struct TodoListMutation;

#[Object]
impl TodoListMutation {
    async fn todo_list_create(&self, ctx: &Context<'_>, input: TodoListInputCreate) -> Result<ID> {
        let id = ID::new(TODO_LIST_KIND);
        let dynamodb = ctx.data_unchecked::<DynamoDB>();
        dynamodb.put_item(&id, &id, |req| {
            req.item("title", AttributeValue::S(input.title.clone()))
        }).await?;
        Ok(id)
    }

    async fn todo_list_update(&self, ctx: &Context<'_>, input: TodoListInputUpdate) -> Result<TodoList> {
        let dynamodb = ctx.data_unchecked::<DynamoDB>();

        if let Some(title) = input.title {
            dynamodb.update_item(&input.id, &input.id, |req| {
                req.update_expression("set title = :title")
                    .expression_attribute_values(":title", AttributeValue::S(title))
            }).await?;
        }

        // TODO: force consistent read
        TodoListQuery.todo_list(ctx, input.id).await
    }

    async fn todo_list_delete(&self, ctx: &Context<'_>, id: ID) -> Result<bool> {
        let dynamodb = ctx.data_unchecked::<DynamoDB>();
        dynamodb.delete_item(&id, &id, identity).await?;
        Ok(true)
    }
}
