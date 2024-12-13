use core::fmt;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Debug)]
pub struct Node {
    pub id: String,
    pub label: String,
    pub properties: HashMap<String, Value>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Edge {
    pub id: String,
    pub label: String,
    pub from_node: String,
    pub to_node: String,
    pub properties: HashMap<String, Value>,
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub enum Value {
    String(String),
    Float(f64),
    Integer(i32),
    Boolean(bool),
    Array(Vec<Value>),
    Null,
}

#[derive(Debug)]
pub enum GraphError {
    Io(std::io::Error),
    GraphConnectionError(String, std::io::Error),
    StorageConnectionError(String, std::io::Error),
    New(String)
}

impl fmt::Display for GraphError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            GraphError::Io(e) => write!(f, "IO error: {}", e),
            GraphError::StorageConnectionError(msg, e) => {
                write!(f, "Error: {}", format!("{} {}", msg, e))
            },
            GraphError::GraphConnectionError(msg, e) => {
                write!(f, "Error: {}", format!("{} {}", msg, e))
            },
            GraphError::New(msg) => write!(f, "Graph error: {}", msg),
        }
    }
}

impl From<rocksdb::Error> for GraphError {
    fn from(error: rocksdb::Error) -> Self {
        GraphError::New(error.into_string())
    }
}

