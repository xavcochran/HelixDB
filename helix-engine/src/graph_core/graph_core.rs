use std::str;
use crate::types::GraphError;
use crate::HelixGraphStorage;

use super::traversal::TraversalBuilder;
use serde_json::json;
pub struct HelixGraphEngine {
    pub storage: HelixGraphStorage,
}

impl HelixGraphEngine {
    pub fn new(path: &str) -> Result<HelixGraphEngine, GraphError> {
        let storage = match HelixGraphStorage::new(path) {
            Ok(db) => db,
            Err(err) => return Err(err),
        };
        Ok(Self { storage })
    }

    pub fn result_to_json(&self, traversal: &TraversalBuilder) {
        let current_step = &traversal.current_step;
        let json_result = json!(current_step);
        println!("{}", json_result.to_string());
    }
}