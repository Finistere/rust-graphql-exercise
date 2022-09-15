use std::convert::identity;

use async_graphql::Result;
use aws_sdk_dynamodb::model::{AttributeValue, ReturnValue};
use tokio_stream::StreamExt;

use crate::dynamodb::{AttributesGetterExt, DynamoTable, RawAttributes};
use crate::graphql::types::ID;
use crate::graphql::Key;

use super::{Todo, TODO_TYPE_NAME};

/// Extension used to decorate the DynamoTable with specialized methods for Todo
#[async_trait::async_trait]
pub trait DynamoTableTodoExt {
    async fn scan_todo(&self) -> Result<Vec<Todo>>;
    async fn get_todo(&self, id: &ID) -> Result<Option<(Key, Todo)>>;
    async fn put_todo(&self, todo: &Todo) -> Result<bool>;
    async fn update_todo(
        &self,
        old_key: Key,
        old_todo: Todo,
        new_todo: Todo,
    ) -> Result<(Key, Todo)>;
    async fn delete_todo(&self, id: &ID) -> Result<Option<Todo>>;
}

#[async_trait::async_trait]
impl DynamoTableTodoExt for DynamoTable {
    async fn scan_todo(&self) -> Result<Vec<Todo>> {
        let mut todos: Vec<Todo> = Vec::new();
        let mut paginator = self
            .scan()
            .filter_expression("begins_with(#sk, :sk)")
            .expression_attribute_names("#sk", &self.config.sort_key)
            .expression_attribute_values(":sk", AttributeValue::S(ID::prefix(TODO_TYPE_NAME)))
            .into_paginator()
            .send();

        while let Some(output) = paginator.next().await {
            for item in output?.items().unwrap_or_default() {
                let pkey: ID = item.get_from_string(&self.config.gsi1_partition_key)?;
                let skey: ID = item.get_from_string(&self.config.gsi1_partition_key)?;
                let list_id = if pkey != skey { Some(pkey) } else { None };
                let todo = Todo {
                    id: skey,
                    title: item.get_string("title")?.clone(),
                    complete: *item.get_bool("complete")?,
                    list_id,
                };
                todos.push(todo);
            }
        }
        Ok(todos)
    }

    async fn get_todo(&self, id: &ID) -> Result<Option<(Key, Todo)>> {
        let gsi1_key = Key {
            partition: id.clone(),
            sort: id.clone(),
        };
        let output = self.query_gsi1_get(&gsi1_key, identity).await?;

        Ok(if let Some(item) = output.items.unwrap_or_default().pop() {
            let key = self.extract_key(&item)?;
            let todo = build_todo(&key, &item)?;
            Some((key, todo))
        } else {
            None
        })
    }

    async fn put_todo(&self, todo: &Todo) -> Result<bool> {
        let key = Key {
            partition: todo.list_id.clone().unwrap_or_else(|| todo.id.clone()),
            sort: todo.id.clone(),
        };
        self.put_item(&key, |put| {
            put.item("title", AttributeValue::S(todo.title.clone()))
                .item("complete", AttributeValue::Bool(todo.complete))
                // Even if associated with a todo_list, we can retrieve it directly through
                // the secondary index.
                .item(
                    &self.config.gsi1_partition_key,
                    AttributeValue::S(String::from(&todo.id)),
                )
                .item(
                    &self.config.gsi1_sort_key,
                    AttributeValue::S(String::from(&todo.id)),
                )
        })
        .await?;
        Ok(true)
    }

    async fn update_todo(
        &self,
        old_key: Key,
        old_todo: Todo,
        new_todo: Todo,
    ) -> Result<(Key, Todo)> {
        if old_todo.list_id == new_todo.list_id {
            update_todo_inplace(self, old_key, old_todo, new_todo).await
        } else {
            let new_key = Key {
                partition: new_todo
                    .list_id
                    .clone()
                    .unwrap_or_else(|| old_todo.id.clone()),
                sort: old_key.sort.clone(),
            };
            move_todo(self, old_key, new_key, new_todo).await
        }
    }

    async fn delete_todo(&self, id: &ID) -> Result<Option<Todo>> {
        let gsi1_key = Key {
            partition: id.clone(),
            sort: id.clone(),
        };
        let query_output = self.query_gsi1_get(&gsi1_key, identity).await?;
        Ok(
            if let Some(item) = query_output.items.unwrap_or_default().pop() {
                let key = self.extract_key(&item)?;
                let delete_output = self
                    .delete_item(&key, |req| req.return_values(ReturnValue::AllOld))
                    .await?;
                if let Some(item) = delete_output.attributes {
                    Some(build_todo(&key, &item)?)
                } else {
                    None
                }
            } else {
                None
            },
        )
    }
}

//
// utilities
//

async fn move_todo(
    dynamodb: &DynamoTable,
    old_key: Key,
    new_key: Key,
    new_todo: Todo,
) -> Result<(Key, Todo)> {
    dynamodb
        .transact_write(|transaction| {
            transaction
                .transact_items(dynamodb.transact_delete(&old_key, identity))
                .transact_items(dynamodb.transact_put(&new_key, |put| {
                    put.item("title", AttributeValue::S(new_todo.title.clone()))
                        .item("complete", AttributeValue::Bool(new_todo.complete))
                        .item(
                            &dynamodb.config.gsi1_partition_key,
                            AttributeValue::S(String::from(&new_todo.id)),
                        )
                        .item(
                            &dynamodb.config.gsi1_sort_key,
                            AttributeValue::S(String::from(&new_todo.id)),
                        )
                }))
        })
        .await?;
    Ok((new_key, new_todo))
}

async fn update_todo_inplace(
    dynamodb: &DynamoTable,
    key: Key,
    old_todo: Todo,
    new_todo: Todo,
) -> Result<(Key, Todo)> {
    let resp = dynamodb
        .update_item(&key, |mut req| {
            req = req.return_values(ReturnValue::AllNew);

            // Is there a better syntax?
            let mut update_expression: Vec<&str> = vec![];
            req = if old_todo.title != new_todo.title {
                update_expression.push(" title = :title ");
                req.expression_attribute_values(":title", AttributeValue::S(new_todo.title))
            } else {
                req
            };

            req = if old_todo.complete != new_todo.complete {
                update_expression.push(" complete = :complete ");
                req.expression_attribute_values(
                    ":complete",
                    AttributeValue::Bool(new_todo.complete),
                )
            } else {
                req
            };

            if !update_expression.is_empty() {
                req.update_expression("SET ".to_string() + &update_expression.join(" , "))
            } else {
                req
            }
        })
        .await?;

    let item = resp
        .attributes()
        .ok_or_else(|| anyhow::anyhow!("Missing attributes"))?;
    let todo = build_todo(&key, item)?;
    Ok((key, todo))
}

fn build_todo(key: &Key, item: &RawAttributes) -> Result<Todo> {
    let list_id = if key.partition != key.sort {
        Some(key.partition.clone())
    } else {
        None
    };
    Ok(Todo {
        id: key.sort.clone(),
        title: item.get_string("title")?.clone(),
        complete: *item.get_bool("complete")?,
        list_id,
    })
}
