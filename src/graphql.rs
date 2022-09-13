use async_graphql::{EmptySubscription, MergedObject, Schema};
use async_graphql::extensions::Tracing;

use crate::DynamoDB;
use crate::graphql::todo::{TodoMutation, TodoQuery};
use crate::graphql::todo_list::{TodoListMutation, TodoListQuery};

mod errors;
mod todo;
mod todo_list;
pub mod id;

pub type GraphQLSchema = Schema<Query, Mutation, EmptySubscription>;

#[derive(MergedObject, Default)]
pub struct Query(TodoQuery, TodoListQuery);

#[derive(MergedObject, Default)]
pub struct Mutation(TodoMutation, TodoListMutation);


pub fn build_schema(db: DynamoDB) -> GraphQLSchema {
    Schema::build(Query::default(), Mutation::default(), EmptySubscription)
        .extension(Tracing)
        .data(db)
        .finish()
}
