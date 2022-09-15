use std::convert::identity;

use async_graphql::{Error, Result};
use aws_sdk_dynamodb::model::{AttributeValue, ReturnValue};
use tokio_stream::StreamExt;

use crate::dynamodb::{AttributesGetterExt, RawAttributes};
use crate::graphql::model::todo::{Todo, TODO_KIND};
use crate::graphql::types::id::{Kind, ID};
use crate::graphql::Key;
use crate::DynamoTable;

use super::TodoList;

#[async_trait::async_trait]
pub trait DynamoTableTodoListExt {
    async fn get_todo_list_todos(&self, id: &ID) -> Result<Vec<Todo>>;
    async fn get_todo_list(&self, id: &ID) -> Result<Option<TodoList>>;
    async fn put_todo_list(&self, todo_list: &TodoList) -> Result<bool>;
    async fn update_todo_list(&self, id: &ID, new_title: String) -> Result<TodoList>;
    async fn delete_todo_list(&self, id: &ID) -> Result<Option<TodoList>>;
}

#[async_trait::async_trait]
impl DynamoTableTodoListExt for DynamoTable {
    async fn get_todo_list_todos(&self, id: &ID) -> Result<Vec<Todo>> {
        let mut todos: Vec<Todo> = Vec::new();
        let todo_kind = Kind::from_string(TODO_KIND);
        let mut paginator = self
            .query_partition_by_prefix(&id, &ID::prefix(&todo_kind))
            .into_paginator()
            .send();
        while let Some(output) = paginator.next().await {
            for item in output?.items().unwrap_or_default() {
                let todo = Todo {
                    id: item.get_from_string(&self.config.gsi1_partition_key)?,
                    title: item.get_string("title")?.clone(),
                    complete: item.get_bool("complete")?.clone(),
                    list_id: Some(id.clone()),
                };
                todos.push(todo);
            }
        }
        Ok(todos)
    }

    async fn get_todo_list(&self, id: &ID) -> Result<Option<TodoList>> {
        let key = Key {
            partition: id.clone(),
            sort: id.clone(),
        };
        let output = self.get_item(&key, identity).await?;
        Ok(if let Some(item) = output.item {
            Some(build_todo_list(&id, &item)?)
        } else {
            None
        })
    }

    async fn put_todo_list(&self, todo_list: &TodoList) -> Result<bool> {
        let key = Key {
            partition: todo_list.id.clone(),
            sort: todo_list.id.clone(),
        };

        self.put_item(&key, |req| {
            req.item("title", AttributeValue::S(todo_list.title.clone()))
        })
        .await?;
        Ok(true)
    }

    async fn update_todo_list(&self, id: &ID, new_title: String) -> Result<TodoList> {
        let key = Key {
            partition: id.clone(),
            sort: id.clone(),
        };
        let output = self
            .update_item(&key, |req| {
                req.update_expression("SET title = :title")
                    .expression_attribute_values(":title", AttributeValue::S(new_title))
                    .return_values(ReturnValue::AllNew)
            })
            .await?;
        if let Some(item) = output.attributes {
            Ok(build_todo_list(&id, &item)?)
        } else {
            Err(Error::new("Missing attributes"))
        }
    }

    async fn delete_todo_list(&self, id: &ID) -> Result<Option<TodoList>> {
        let key = Key {
            partition: id.clone(),
            sort: id.clone(),
        };
        let output = self
            .delete_item(&key, |req| req.return_values(ReturnValue::AllOld))
            .await?;
        Ok(if let Some(item) = output.attributes {
            Some(build_todo_list(&id, &item)?)
        } else {
            None
        })
    }
}

fn build_todo_list(id: &ID, item: &RawAttributes) -> Result<TodoList> {
    Ok(TodoList {
        id: id.clone(),
        title: item.get_string("title")?.clone(),
    })
}
