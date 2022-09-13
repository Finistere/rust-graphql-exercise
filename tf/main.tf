terraform {
  required_version = ">= 0.12"
  required_providers {
    aws = {
      source  = "hashicorp/aws"
      version = "~> 4.16"
    }
  }

  backend "s3" {
    region         = "eu-west-3"
    bucket         = "brabier-rust-graphql-exercise-terraform-state"
    key            = "terraform.tfstate"
    dynamodb_table = "rust-graphql-exercise-terraform-state"
  }
}

provider "aws" {
  region = "eu-west-3" # Paris
}