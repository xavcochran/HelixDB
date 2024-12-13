use std::str;

use crate::graph_core::graph_methods::GraphMethods;
use crate::types::{Edge, GraphError, Node, Value};
use crate::HelixGraphStorage;
pub struct HelixGraphEngine {
    storage: HelixGraphStorage,
}

impl HelixGraphEngine {
    pub fn new(path: &str) -> Result<HelixGraphEngine, GraphError> {
        let storage = match HelixGraphStorage::new(path) {
            Ok(db) => db,
            Err(err) => return Err(err),
        };
        Ok(Self { storage })
    }
}