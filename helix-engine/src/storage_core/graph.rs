use core::fmt;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize)]
pub struct Node {
    pub id: String,
    pub label: String,
    pub properties: HashMap<String, Value>,
}

#[derive(Serialize, Deserialize)]
pub struct Edge {
    pub id: String,
    pub label: String,
    pub from_node: String,
    pub to_node: String,
    pub properties: HashMap<String, Value>,
}

#[derive(Serialize, Deserialize)]
pub enum Value {
    String(String),
    Number(f64),
    Boolean(bool),
    Array(Vec<Value>),
    Null,
}

pub trait GraphMethods {
    fn check_exists(&self, id: &str) -> Result<bool, GraphError>;

    fn get_node(&self, id: &str) -> Result<Node, GraphError>;
    fn get_edge(&self, id: &str) -> Result<Edge, GraphError>;

    fn get_temp_node(&self, id: &str) -> Result<Node, GraphError>;
    fn get_temp_edge(&self, id: &str) -> Result<Edge, GraphError>;

    fn create_node(
        &self,
        label: &str,
        properties: HashMap<String, Value>,
    ) -> Result<Node, GraphError>;
    fn create_edge(
        &self,
        label: &str,
        from_node: &str,
        to_node: &str,
        properties: HashMap<String, Value>,
    ) -> Result<Edge, GraphError>;

    fn drop_node(&self, id: &str) -> Result<(), GraphError>;
    fn drop_edge(&self, id: &str) -> Result<(), GraphError>;
}

#[derive(Debug)]
pub enum GraphError {
    Io(std::io::Error),
    Other(String),
    ConnectionError(String, std::io::Error),
}

impl fmt::Display for GraphError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            GraphError::Io(e) => write!(f, "IO error: {}", e),
            GraphError::Other(msg) => write!(f, "Error: {}", msg),
            GraphError::ConnectionError(msg, e) => {
                write!(f, "Error: {}", format!("{} {}", msg, e))
            }
        }
    }
}

impl From<rocksdb::Error> for GraphError {
    fn from(error: rocksdb::Error) -> Self {
        GraphError::Other(error.into_string())
    }
}
