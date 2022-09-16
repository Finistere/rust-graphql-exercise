# Rust GraphQL Exercise

The current repository implements the following model on top of AWS DynamoDB.

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


## Setup

Terraform is available in the `tf` folder which creates a new DynamoDB table. The Terraform state is stored inside a S3
bucket with object lock and encryption.

The application has several configuration parameters defined in `App.toml` and can be run with:

```shell
cargo run
```

Logs are generated in the Bunyan format, so `bunyan` can be used to generate friendlier messages:

```shell
cargo run | bunyan -l warn
```


## Data Model

Everything is stored in a single DynamoDB table:

```text
+--------------+--------------+--------------+--------------+
|     PK       |      SK      |   GSI1-PK    |   GSI1-SK    |
+--------------+--------------+--------------+--------------+
| todo#ID      | todo#ID      | todo#ID      | todo#ID      |  <- standalone Todo
| todo_list#ID | todo_list#ID |              |              |  <- TodoList
| todo_list#ID | todo#ID      | todo#ID      | todo#ID      |  <- Todo associated with a TodoList
+--------------+--------------+--------------+--------------+
```

Here are the main access patterns:

1) retrieve a `Todo` by its `id`: `GSI1-PK  = 'todo#ID'`
2) retrieve a `TodoList` by its `id`: `PK = 'todo_list#ID'`
3) retrieve all `Todo`s of a `TodoList`: `PK = 'todo_list#ID' and begins_with(GSI1-PK, 'todo#')`
4) retrieve all `Todo` (or `TodoList`): Scan with `begins_with(SK, 'todo#')`

The global secondary index `GSI1` includes all attributes mainly for simplicity reasons.

## GraphQL

The application exposes the following schema:

```graphql
scalar Id

type Mutation {
  todoCreate(input: TodoCreateInput!): Todo!
  todoUpdate(input: TodoUpdateInput!): Todo!
  todoDelete(id: Id!): Todo!
  todoListCreate(input: TodoListInputCreate!): TodoList!
  todoListUpdate(input: TodoListInputUpdate!): TodoList!
  todoListDelete(id: Id!): TodoList!
}

type Query {
  todoCollection: [Todo!]!
  todo(id: Id!): Todo!
  todoListCollection: [TodoList!]!
  todoList(id: Id!): TodoList!
}

type Todo {
  id: Id!
  title: String!
  complete: Boolean!
  list: TodoList
}

input TodoCreateInput {
  title: String!
  complete: Boolean
  list: TodoRelationTodoListLinkInput
}

type TodoList {
  id: Id!
  title: String!
  todos: [Todo!]!
}

input TodoListInputCreate {
  title: String!
}

input TodoListInputUpdate {
  id: Id!
  title: String
}

input TodoRelationTodoListLinkInput {
  link: Id!
}

input TodoRelationTodoListUpdateInput {
  link: Id
  unlink: Id
}

input TodoUpdateInput {
  id: Id!
  title: String
  list: TodoRelationTodoListUpdateInput
  complete: Boolean
}
```

## Tests

All tests were done by hand... For a real production project I would focus on functional tests. I would start the 
application locally with a [LocalStack](https://docs.localstack.cloud/overview/) docker image imitating DynamoDB and
execute GraphQL requests. I'm not sure in which language I would write those tests though, either in Rust or in
TypeScript as the tooling to generate GraphQL clients might be better. The project also obviously needs proper
CI (clippy, format, test...).

## Documentation

Pretty obvious, but it's worth mentioning that more documentation could not hurt especially on the GraphQL API if it 
was to be used by a client.

## Technical Limitations & Possible Improvements

- The global secondary index currently includes all attributes. Using `KEYS_ONLY` would generate a smaller index but
  would increase the complexity in the codebase. Without trying it out I'm also unsure on the actual cost/performance 
  impact. Including some attributes only some attributes is obviously an intermediate solution. 
- The collection endpoints should use a Relay style cursor API instead of retrieving everything at once.
- Updating a `Todo` with a different `TodoList` isn't an atomic operation. It will first fetch the data and then in a 
  transaction change it. Any changes between those requests will not be taken into account. To circumvent this we could
  use an atomic counter `item_version`, incremented on update, and a condition check inside the transaction. If the 
  `item_version` changed, the transaction is aborted.
- DynamoDB requests could be batched together with DataLoaders.
- To handle proper data model migration I would add a `schema_version` attribute which can be used to know if an item
  needs to be migrated or not
- I wondered whether the keys should be stored in binary or not. It would improve space efficiency, but it implies having
  to create/use proper tooling for data exploration and exports to show keys in a friendlier format. Here I choose to
  store them as strings for the sake of simplicity.

