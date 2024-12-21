
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub mod request;
pub mod response;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Node {
    pub id: String,
    pub label: String,
    pub properties: HashMap<String, Value>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Edge {
    pub id: String,
    pub label: String,
    pub from_node: String,
    pub to_node: String,
    pub properties: HashMap<String, Value>,
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub enum Value {
    String(String),
    Float(f64),
    Integer(i32),
    Boolean(bool),
    Array(Vec<Value>),
    Null,
}

impl From<&str> for Value {
    fn from(s: &str) -> Self {
        Value::String(s.to_string())
    }
}

impl From<i32> for Value {
    fn from(i: i32) -> Self {
        Value::Integer(i)
    }
}

impl From<bool> for Value {
    fn from(b: bool) -> Self {
        Value::Boolean(b)
    }
}