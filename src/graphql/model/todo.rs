use async_graphql::{ComplexObject, Context, Result, SimpleObject};

use crate::dynamodb::DynamoTable;
use crate::graphql::model::todo_list::extensions::DynamoTableTodoListExt;
use crate::graphql::model::TodoList;
use crate::graphql::types::ID;

pub mod extensions;
pub mod mutation;
pub mod query;

pub const TODO_TYPE_NAME: &str = "todo";

#[derive(Debug, SimpleObject)]
#[graphql(complex)]
pub struct Todo {
    pub id: ID,
    pub title: String,
    pub complete: bool,
    #[graphql(skip)]
    pub list_id: Option<ID>,
}

#[ComplexObject]
impl Todo {
    async fn list(&self, ctx: &Context<'_>) -> Result<Option<TodoList>> {
        if let Some(id) = self.list_id.clone() {
            let dynamodb = ctx.data_unchecked::<DynamoTable>();
            dynamodb.get_todo_list(&id).await
        } else {
            Ok(None)
        }
    }
}
