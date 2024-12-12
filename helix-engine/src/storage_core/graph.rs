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
    /// Checks whether an entry with a given id exists.
    /// Works for nodes or edges.
    fn check_exists(&self, id: &str) -> Result<bool, GraphError>;

    /// Gets a node object for a given node id
    fn get_node(&self, id: &str) -> Result<Node, GraphError>;
    /// Gets a edge object for a given edge id
    fn get_edge(&self, id: &str) -> Result<Edge, GraphError>;

    /// Gets a node object for a given node id without copying its underlying data. 
    /// 
    /// This should only used when fetched data is only needed temporarily
    /// as underlying data is pinned.
    fn get_temp_node(&self, id: &str) -> Result<Node, GraphError>;

    /// Gets a edge object for a given edge id without copying its underlying data. 
    /// 
    /// This should only used when fetched data is only needed temporarily
    /// as underlying data is pinned.
    fn get_temp_edge(&self, id: &str) -> Result<Edge, GraphError>;

    /// Creates a node entry
    fn create_node(
        &self,
        label: &str,
        properties: HashMap<String, Value>,
    ) -> Result<Node, GraphError>;

    /// Creates an edge entry between two nodes
    fn create_edge(
        &self,
        label: &str,
        from_node: &str,
        to_node: &str,
        properties: HashMap<String, Value>,
    ) -> Result<Edge, GraphError>;

    /// Deletes a node entry along with all of its connected edges 
    fn drop_node(&self, id: &str) -> Result<(), GraphError>;

    /// Deletes an edge entry
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
