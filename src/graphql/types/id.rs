use std::fmt::{Display, Formatter};
use std::str::FromStr;

use anyhow::anyhow;
use async_graphql::{InputValueError, InputValueResult, Scalar, ScalarType, Value};
use ulid::Ulid;

#[derive(Debug, Clone, PartialEq)]
pub struct Kind(String);

#[derive(Debug, Clone, PartialEq)]
pub struct ID {
    kind: Kind,
    ulid: Ulid,
}

const SEPARATOR: &str = "#";

impl Kind {
    pub fn from_string(kind: &str) -> Kind {
        Kind(String::from(kind))
    }
}

impl ID {
    pub fn new(kind: &str) -> ID {
        ID {
            kind: Kind::from_string(kind),
            ulid: Ulid::new(),
        }
    }

    pub fn prefix(kind: &Kind) -> String {
        format!("{}{}", kind.0, SEPARATOR)
    }

    pub fn has_kind(&self, kind: &str) -> bool {
        self.kind.0 == kind
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
        if let Some((kind, ulid)) = s.split_once(SEPARATOR) {
            Ok(ID {
                kind: Kind::from_string(kind),
                ulid: Ulid::from_string(ulid)?,
            })
        } else {
            Err(anyhow!("Invalid ID format"))
        }
    }
}

impl From<&ID> for String {
    fn from(id: &ID) -> Self {
        format!("{}{}{}", id.kind.0, SEPARATOR, id.ulid)
    }
}

#[Scalar]
impl ScalarType for ID {
    fn parse(value: Value) -> InputValueResult<Self> {
        if let Value::String(value) = &value {
            value.parse().map_err(|e| InputValueError::from(e))
        } else {
            Err(InputValueError::expected_type(value))
        }
    }

    fn to_value(&self) -> Value {
        Value::String(String::from(self))
    }
}
