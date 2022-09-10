use async_graphql::{Object, SimpleObject};

struct Query;

type ID = String;

#[Object]
impl Query {
    /// Returns the sum of a and b
    async fn todo_create(&self, id: ID, title: String, complete: bool) -> bool {
        println!("{:?}", Todo { id, title, complete });
        true
    }
}

#[derive(Debug, SimpleObject)]
struct Todo {
    id: ID,
    title: String,
    complete: bool
}