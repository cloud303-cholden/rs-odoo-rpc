use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Credentials<S>
where
    S: AsRef<str> + std::fmt::Debug,
{
    pub db: S,
    pub username: S,
    pub password: S,
    pub url: S,
}

#[derive(Clone, Debug, Deserialize)]
pub struct Response<T> {
    // #[serde(bound(deserialize = "Vec<T>: Deserialize<'de>"))]
    pub result: ArrayOrAny<T>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(untagged)]
pub enum OdooOption<T> {
    Bool(bool),
    #[serde(bound(deserialize = "T: Deserialize<'de>"))]
    Value(T),
}

#[allow(clippy::from_over_into)]
impl<T> Into<Option<T>> for OdooOption<T> {
    fn into(self) -> Option<T> {
        match self {
            Self::Bool(_) => None,
            Self::Value(val) => Some(val),
        }
    }
}

#[derive(Clone, Debug, Deserialize)]
#[serde(untagged)]
pub enum ArrayOrNumber {
    Array(Vec<u64>),
    Number(u64),
}

#[derive(Clone, Debug, Deserialize)]
#[serde(untagged)]
pub enum ArrayOrAny<T> {
    #[serde(bound(deserialize = "Vec<T>: Deserialize<'de>"))]
    Array(Vec<T>),
    #[serde(bound(deserialize = "T: Deserialize<'de>"))]
    Any(T),
}

#[allow(clippy::from_over_into)]
impl<T> Into<Vec<T>> for ArrayOrAny<T> {
    fn into(self) -> Vec<T> {
        match self {
            Self::Array(val) => val,
            Self::Any(val) => vec![val],
        }
    }
}

impl<T> ArrayOrAny<T> {
    pub fn to_records(self) -> Vec<T> {
        self.into()
    }
}

#[allow(clippy::from_over_into)]
impl Into<Vec<u64>> for ArrayOrNumber {
    fn into(self) -> Vec<u64> {
        match self {
            Self::Array(val) => val,
            Self::Number(val) => vec![val],
        }
    }
}

impl From<u64> for ArrayOrNumber {
    fn from(value: u64) -> Self {
        Self::Number(value)
    }
}

impl From<Vec<u64>> for ArrayOrNumber {
    fn from(value: Vec<u64>) -> Self {
        Self::Array(value)
    }
}
