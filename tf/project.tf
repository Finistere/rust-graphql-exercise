resource "aws_dynamodb_table" "graphql" {
  name           = var.dynamodb_table_name
  billing_mode   = "PROVISIONED"
  read_capacity  = 1
  write_capacity = 1
  hash_key       = "PK"
  range_key      = "SK"

  attribute {
    name = "PK"
    type = "S"
  }

  attribute {
    name = "SK"
    type = "S"
  }

  attribute {
    name = "GSI1-PK"
    type = "S"
  }

  attribute {
    name = "GSI1-SK"
    type = "S"
  }

  global_secondary_index {
    name               = "GSI1"
    hash_key           = "GSI1-PK"
    range_key          = "GSI1-SK"
    write_capacity     = 1
    read_capacity      = 1
    # For the sake of simplicity, we're simply including all attributes in this example.
    projection_type    = "ALL"
  }
}
