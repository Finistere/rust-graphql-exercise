#
# Bucket used to store the tfstate file.
#

resource "aws_s3_bucket" "terraform-state" {
  bucket = "brabier-rust-graphql-exercise-terraform-state"
  # Ensuring tfstate files cannot be deleted for a certain time
  object_lock_enabled = true
}

resource "aws_s3_bucket_object_lock_configuration" "terraform-state-bucket-lock" {
  bucket = aws_s3_bucket.terraform-state.id

  rule {
    default_retention {
      mode = "COMPLIANCE"
      days = 5
    }
  }
}

#
# Ensuring no public access to the bucket
#

resource "aws_s3_bucket_acl" "terraform-state-acl" {
  bucket = aws_s3_bucket.terraform-state.id
  acl    = "private"
}

resource "aws_s3_bucket_public_access_block" "terraform-state-bucket-block" {
  bucket = aws_s3_bucket.terraform-state.id

  block_public_acls       = true
  block_public_policy     = true
  ignore_public_acls      = true
  restrict_public_buckets = true
}

#
# Server-side encryption
#

resource "aws_kms_key" "terraform-bucket-key" {
  description             = "This key is used to encrypt bucket objects"
  deletion_window_in_days = 10
  enable_key_rotation     = true
}

resource "aws_kms_alias" "terraform-bucket-key-alias" {
  name          = "alias/rust-graphql-exercise-terraform-bucket-key"
  target_key_id = aws_kms_key.terraform-bucket-key.key_id
}

resource "aws_s3_bucket_server_side_encryption_configuration" "terraform-state-bucket-encryption" {
  bucket = aws_s3_bucket.terraform-state.id

  rule {
    apply_server_side_encryption_by_default {
      kms_master_key_id = aws_kms_key.terraform-bucket-key.arn
      sse_algorithm     = "aws:kms"
    }
  }
}

#
# DynamoDB table used for locking purposes.
#

resource "aws_dynamodb_table" "terraform-lock" {
  name           = "rust-graphql-exercise-terraform-state"
  read_capacity  = 1
  write_capacity = 1
  hash_key       = "LockID"
  attribute {
    name = "LockID"
    type = "S"
  }
}
