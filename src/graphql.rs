use async_graphql::extensions::Tracing;
use async_graphql::{EmptySubscription, MergedObject, Schema};

use model::todo::mutation::TodoMutation;
use model::todo::query::TodoQuery;
use model::todo_list::mutation::TodoListMutation;
use model::todo_list::query::TodoListQuery;

use crate::dynamodb::ItemKey;
use crate::graphql::types::id::ID;
use crate::DynamoTable;

mod errors;
mod model;
mod types;

pub type GraphQLSchema = Schema<Query, Mutation, EmptySubscription>;
type Key = ItemKey<ID>;

#[derive(MergedObject, Default)]
pub struct Query(TodoQuery, TodoListQuery);

#[derive(MergedObject, Default)]
pub struct Mutation(TodoMutation, TodoListMutation);

pub fn build_schema(db: DynamoTable) -> GraphQLSchema {
    Schema::build(Query::default(), Mutation::default(), EmptySubscription)
        .extension(Tracing)
        .data(db)
        .finish()
}
