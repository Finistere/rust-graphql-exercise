use std::collections::HashMap;
use std::convert::identity;

use async_graphql::{ComplexObject, Context, ErrorExtensions, InputObject, Object, OneofObject, Result, SimpleObject};
use aws_sdk_dynamodb::model::AttributeValue;

use crate::DynamoDB;
use crate::dynamodb::AttributesGetterExt;
use crate::graphql::errors::Errors;
use crate::graphql::id::ID;
use crate::graphql::todo_list::{TodoList, TodoListQuery};

pub const TODO_KIND: &str = "todo";

#[derive(Debug, SimpleObject)]
#[graphql(complex)]
pub struct Todo {
    id: ID,
    title: String,
    complete: bool,
    #[graphql(skip)]
    list_id: Option<ID>,
}

#[ComplexObject]
impl Todo {
    async fn list(&self, ctx: &Context<'_>) -> Result<Option<TodoList>> {
        Ok(
            if let Some(id) = self.list_id.clone() {
                Some(TodoListQuery.todo_list(ctx, id).await?)
            } else { None }
        )
    }

    #[graphql(skip)]
    pub fn try_from_attributes(id: ID, maybe_item: Option<&HashMap<String, AttributeValue>>) -> Result<Todo> {
        if let Some(item) = maybe_item {
            Ok(Todo {
                id,
                title: item.get_string("title")?.clone(),
                complete: item.get_bool("complete")?.clone(),
                list_id: None,
            })
        } else {
            Err(Errors::NotFound.extend())
        }
    }
}

//
// QUERY
//

#[derive(Default)]
pub struct TodoQuery;

#[Object]
impl TodoQuery {
    async fn todo(&self, ctx: &Context<'_>, id: ID) -> Result<Todo> {
        let dynamodb = ctx.data_unchecked::<DynamoDB>();
        let output = dynamodb.get_item(&id, &id, identity).await?;
        Todo::try_from_attributes(id, output.item())
    }
}

//
// MUTATION
//

#[derive(Debug, InputObject)]
struct TodoCreateInput {
    title: String,
    complete: Option<bool>,
    list: Option<TodoRelationTodoListLinkInput>,
}

#[derive(Debug, InputObject)]
struct TodoUpdateInput {
    id: ID,
    title: Option<String>,
    link: Option<TodoRelationTodoListUpdateInput>,
    complete: Option<bool>,
}

#[derive(Debug, OneofObject)]
enum TodoRelationTodoListUpdateInput {
    Link(TodoRelationTodoListLinkInput),
    Unlink(TodoRelationTodoListUnlinkInput),
}

#[derive(Debug, InputObject)]
struct TodoRelationTodoListLinkInput {
    link: ID,
}

#[derive(Debug, InputObject)]
struct TodoRelationTodoListUnlinkInput {
    unlink: ID,
}

#[derive(Default)]
pub struct TodoMutation;

#[Object]
impl TodoMutation {
    async fn todo_create(&self, ctx: &Context<'_>, input: TodoCreateInput) -> Result<ID> {
        let id = ID::new(&TODO_KIND);
        let dynamodb = ctx.data_unchecked::<DynamoDB>();
        // Either the todo_list as primary key or the todo itself.
        let pkey = input.list.map(|link| link.link).unwrap_or(id.clone());
        dynamodb.put_item(&pkey, &id, |put| {
            put
                .item("title", AttributeValue::S(input.title.clone()))
                .item(
                    "complete",
                    AttributeValue::Bool(input.complete.unwrap_or(false)),
                )
                // Even if associated with a todo_list, we can retrieve it directly through
                // the secondary index.
                .item(&dynamodb.config.gsi1_partition_key, AttributeValue::S(String::from(&id)))
                .item(&dynamodb.config.gsi1_sort_key, AttributeValue::S(String::from(&id)))
        }).await?;
        Ok(id)
    }

    async fn todo_update(&self, ctx: &Context<'_>, input: TodoUpdateInput) -> Result<Todo> {
        let dynamodb = ctx.data_unchecked::<DynamoDB>();
        // We need to retrieve the real key of the item, as it might be associated with a todo_list already.
        let query_output = dynamodb.query_gsi1_get(&input.id, &input.id, identity).await?;
        let todo_attrs = query_output
            .items()
            .and_then(|items| items.first())
            .ok_or(Errors::NotFound.extend())?;
        let pkey = todo_attrs.get_id(&dynamodb.config.partition_key)?;
        let skey = todo_attrs.get_id(&dynamodb.config.sort_key)?;
        let title = todo_attrs.get_string("title")?;
        let complete = todo_attrs.get_bool("complete")?;

        if let Some(relation_update) = input.link {
            let partition_key: ID = match relation_update {
                TodoRelationTodoListUpdateInput::Link(TodoRelationTodoListLinkInput { link }) => {
                    link.clone()
                }
                TodoRelationTodoListUpdateInput::Unlink(TodoRelationTodoListUnlinkInput { unlink }) => {
                    if pkey != unlink {
                        return Err(Errors::InvalidValue(format!("Todo '{}' is not linked to the todo list '{}'", input.id, unlink)).extend());
                    }
                    input.id.clone()
                }
            };
            dynamodb.transact_write(|transaction| {
                transaction.transact_items(dynamodb.transact_delete(&pkey, &skey, identity))
                    .transact_items(dynamodb.transact_put(&partition_key, &input.id, |put| {
                        put.item("title", AttributeValue::S(title.clone()))
                            .item("complete", AttributeValue::Bool(*complete))
                            .item(&dynamodb.config.gsi1_partition_key, AttributeValue::S(String::from(&input.id)))
                            .item(&dynamodb.config.gsi1_sort_key, AttributeValue::S(String::from(&partition_key)))
                    }))
            }).await?;
        } else {
            // Performing "inplace" update
            dynamodb.update_item(&pkey, &skey, |mut req| {
                // Is there a better syntax?
                let mut update_expression: Vec<&str> = vec![];
                req = match input.title {
                    Some(title) => {
                        update_expression.push(" title = :title ");
                        req.expression_attribute_values(":title", AttributeValue::S(title))
                    }
                    None => req
                };
                req = match input.complete {
                    Some(complete) => {
                        update_expression.push(" complete = :complete ");
                        req.expression_attribute_values(":complete", AttributeValue::Bool(complete))
                    }
                    None => req
                };

                if !update_expression.is_empty() {
                    req.update_expression("SET ".to_string() + &update_expression.join(" , "))
                } else {
                    req
                }
            }).await?;
        }

        // Forcing consistent read to avoid ghost read.
        let output = dynamodb.get_item(&input.id, &input.id, |req| {
            req.consistent_read(true)
        }).await?;
        Todo::try_from_attributes(input.id, output.item())
    }

    async fn todo_delete(&self, ctx: &Context<'_>, id: ID) -> Result<bool> {
        let dynamodb = ctx.data_unchecked::<DynamoDB>();
        dynamodb.delete_item(&id, &id, identity).await?;
        Ok(true)
    }
}
