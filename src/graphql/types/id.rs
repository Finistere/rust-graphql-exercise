use std::fmt::{Display, Formatter};
use std::str::FromStr;

use anyhow::anyhow;
use async_graphql::{InputValueError, InputValueResult, Scalar, ScalarType, Value};
use ulid::Ulid;

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct TypeName(String);

/// Unique identifier across all entities. It will be stored as a string formatted as
/// '<type_name>#<ulid>` in DynamoDB.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct ID {
    type_name: TypeName,
    ulid: Ulid,
}

const SEPARATOR: &str = "#";

impl ID {
    pub fn new(type_name: &str) -> ID {
        ID {
            type_name: TypeName(type_name.to_owned()),
            ulid: Ulid::new(),
        }
    }

    pub fn prefix(type_name: &str) -> String {
        format!("{}{}", type_name, SEPARATOR)
    }

    pub fn has_type_name(&self, type_name: &str) -> bool {
        self.type_name.0 == type_name
    }
}

impl Display for ID {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", String::from(self))
    }
}

impl FromStr for ID {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, anyhow::Error> {
        if let Some((type_name, ulid)) = s.split_once(SEPARATOR) {
            Ok(ID {
                type_name: TypeName(type_name.to_owned()),
                ulid: Ulid::from_string(ulid)?,
            })
        } else {
            Err(anyhow!("Invalid ID format"))
        }
    }
}

impl From<&ID> for String {
    fn from(id: &ID) -> Self {
        format!("{}{}{}", id.type_name.0, SEPARATOR, id.ulid)
    }
}

#[Scalar]
impl ScalarType for ID {
    fn parse(value: Value) -> InputValueResult<Self> {
        if let Value::String(value) = &value {
            value.parse().map_err(InputValueError::from)
        } else {
            Err(InputValueError::expected_type(value))
        }
    }

    fn to_value(&self) -> Value {
        Value::String(String::from(self))
    }
}
