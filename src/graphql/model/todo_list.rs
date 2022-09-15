use async_graphql::{ComplexObject, Context, Result, SimpleObject};

use crate::graphql::model::todo::Todo;
use crate::graphql::model::todo_list::extensions::DynamoTableTodoListExt;
use crate::graphql::types::id::ID;
use crate::DynamoTable;

pub mod extensions;
pub mod mutation;
pub mod query;

pub const TODO_LIST_KIND: &str = "todo_list";

#[derive(Debug, SimpleObject)]
#[graphql(complex)]
pub struct TodoList {
    pub id: ID,
    pub title: String,
}

#[ComplexObject]
impl TodoList {
    async fn todos(&self, ctx: &Context<'_>) -> Result<Vec<Todo>> {
        let dynamodb = ctx.data_unchecked::<DynamoTable>();
        dynamodb.get_todo_list_todos(&self.id).await
    }
}
