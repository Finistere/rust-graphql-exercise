# update
not atomic, but there's no transaction neither. Neither juniper nor async-graphql seems to support transaction.


```graphql
type TodoList @model {
  id: ID!
  title: String!
  todos: [Todo]
}

type Todo @model {
  id: ID!
  title: String!
  complete: Boolean!
  list: TodoList
}
```