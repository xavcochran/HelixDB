use crate::{
    storage_core::{storage_core::HelixGraphStorage, storage_methods::StorageMethods},
    types::{Edge, GraphError, Node, Value},
};
use function_name::named;
use rocksdb::properties;
use std::collections::HashMap;

use super::traversal_steps::{TraversalSteps, SourceTraversalSteps};

#[derive(Debug)]
pub enum TraversalValue {
    SingleNode(Node),
    SingleEdge(Edge),
    SingleValue(Value),
    NodeArray(Vec<Node>),
    EdgeArray(Vec<Edge>),
    ValueArray(Vec<Value>),
}

pub struct TraversalBuilder {
    variables: HashMap<String, TraversalValue>,
    current_step: Vec<TraversalValue>,
}

impl TraversalBuilder {
    pub fn new(start_nodes: Vec<Node>) -> Self {
        let mut builder = Self {
            variables: HashMap::new(),
            current_step: vec![TraversalValue::NodeArray(start_nodes)],
        };
        builder
    }

    pub fn check_is_valid_node_traversal(&self, function_name: &str) -> Result<(), GraphError> {
        match self.current_step.iter().all(|val| {
            matches!(val, TraversalValue::NodeArray(_))
                || matches!(val, TraversalValue::SingleNode(_))
        }) {
            true => Ok(()),
            false => Err(GraphError::TraversalError(format!(
                "The traversal step {:?}, is not a valid traversal from an edge. 
                The current step should be a node.", 
                function_name))),
        }
    }

    pub fn check_is_valid_edge_traversal(&self, function_name: &str) -> Result<(), GraphError> {
        match self.current_step.iter().all(|val| {
            matches!(val, TraversalValue::EdgeArray(_))
                || matches!(val, TraversalValue::SingleEdge(_))
        }) {
            true => Ok(()),
            false => Err(GraphError::TraversalError(format!(
                "The traversal step {:?}, is not a valid traversal from a node. 
                The current step should be an edge", function_name))),
        }
    }

    
}

impl SourceTraversalSteps for TraversalBuilder {
    #[named] 
    fn v(&mut self, storage: &HelixGraphStorage) -> &mut Self {
        self.current_step = vec![];
        self
    }

    fn e(&mut self, storage: &HelixGraphStorage) -> &mut Self {
        self.current_step = vec![];
        self
    }

    #[named]
    fn add_v(&mut self, storage: &HelixGraphStorage, node_label: &str) -> &mut Self {
        let node = storage.create_node(node_label, HashMap::new()).unwrap();
        // TODO: remove hashmap
        self.current_step = vec![TraversalValue::SingleNode(node)];
        self
    }
    
    #[named]
    fn add_e(&mut self, storage: &HelixGraphStorage, edge_label: &str, from_id: &str, to_id: &str) -> &mut Self {
        // TODO: remove hashmap
        let edge = storage.create_edge(edge_label, from_id, to_id, HashMap::new()).unwrap();
        self.current_step = vec![TraversalValue::SingleEdge(edge)];
        self
    }
}

impl TraversalSteps for TraversalBuilder {
    #[named]
    fn out(&mut self, storage: &HelixGraphStorage, edge_label: &str) -> &mut Self {
        self.check_is_valid_node_traversal(function_name!()).unwrap();
            
        if let TraversalValue::NodeArray(nodes) = &self.current_step[0] {
            let mut new_current: Vec<TraversalValue> = Vec::with_capacity(nodes.len());
            let mut next_nodes = Vec::new();
            for node in nodes {
                let edges = storage.get_out_edges(&node.id, edge_label).unwrap();
                for edge in edges {
                    next_nodes.push(storage.get_node(&edge.to_node).unwrap());
                }
            }
            new_current.push(TraversalValue::NodeArray(next_nodes));
            self.current_step = new_current;
        }
        self
    }

    #[named]
    fn out_e(&mut self, storage: &HelixGraphStorage, edge_label: &str) -> &mut Self {
        self.check_is_valid_node_traversal(function_name!()).unwrap();
        if let TraversalValue::NodeArray(nodes) = &self.current_step[0] {
            let mut new_current: Vec<TraversalValue> = Vec::with_capacity(nodes.len());
            for node in nodes {
                let edges = storage.get_out_edges(&node.id, edge_label).unwrap();
                new_current.push(TraversalValue::EdgeArray(edges));
            }
            self.current_step = new_current;
        }
        self
    }

    #[named]
    fn in_(&mut self, storage: &HelixGraphStorage, edge_label: &str) -> &mut Self {
        self.check_is_valid_node_traversal(function_name!()).unwrap();
        if let TraversalValue::NodeArray(nodes) = &self.current_step[0] {
            let mut new_current: Vec<TraversalValue> = Vec::with_capacity(nodes.len());
            let mut next_nodes = Vec::new();
            for node in nodes {
                let edges = storage.get_in_edges(&node.id, edge_label).unwrap();
                for edge in edges {
                    let node_obj = storage.get_node(&edge.to_node).unwrap();
                    next_nodes.push(node_obj);
                }
            }
            new_current.push(TraversalValue::NodeArray(next_nodes));
            self.current_step = new_current;
        }
        self
    }

    #[named]
    fn in_e(&mut self, storage: &HelixGraphStorage, edge_label: &str) -> &mut Self {
        self.check_is_valid_node_traversal(function_name!()).unwrap();
        if let TraversalValue::NodeArray(nodes) = &self.current_step[0] {
            let mut new_current: Vec<TraversalValue> = Vec::with_capacity(nodes.len());
            for node in nodes {
                let edges = storage.get_in_edges(&node.id, edge_label).unwrap();
                new_current.push(TraversalValue::EdgeArray(edges));
            }
            self.current_step = new_current;
        }
        self
    }
}

// need to account for multiple nodes or edges at a given traversal step
//

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use tempfile::TempDir;
    use crate::graph_core::traversal::TraversalBuilder;
    use crate::storage_core::storage_methods::StorageMethods;
    use crate::types::{Node, Edge, Value, GraphError};

    fn setup_test_db() -> (HelixGraphStorage, TempDir) {
        let temp_dir = TempDir::new().unwrap();
        let storage = HelixGraphStorage::new(temp_dir.path().to_str().unwrap()).unwrap();
        (storage, temp_dir)
    }

    #[test]
    fn test_create_and_get_node() {
        let (storage, _temp_dir) = setup_test_db();
        let mut properties = HashMap::new();
        properties.insert("name".to_string(), Value::String("test node".to_string()));
        
        let node = storage.create_node("person", properties).unwrap();
        
        // Test get_node
        let retrieved_node = storage.get_node(&node.id).unwrap();
        assert_eq!(node.id, retrieved_node.id);
        assert_eq!(node.label, "person");
        assert_eq!(
            retrieved_node.properties.get("name").unwrap(),
            &Value::String("test node".to_string())
        );
    }

    #[test]
    fn test_create_and_get_edge() {
        let (storage, _temp_dir) = setup_test_db();
        
        // Create two nodes to connect
        let node1 = storage.create_node("person", HashMap::new()).unwrap();
        let node2 = storage.create_node("person", HashMap::new()).unwrap();
        
        // Create edge
        let mut edge_props = HashMap::new();
        edge_props.insert("weight".to_string(), Value::Float(1.0));
        
        let edge = storage.create_edge(
            "knows",
            &node1.id,
            &node2.id,
            edge_props
        ).unwrap();
        
        // Test get_edge
        let retrieved_edge = storage.get_edge(&edge.id).unwrap();
        assert_eq!(edge.id, retrieved_edge.id);
        assert_eq!(edge.label, "knows");
        assert_eq!(edge.from_node, node1.id);
        assert_eq!(edge.to_node, node2.id);
    }

    #[test]
    fn test_get_out_edges() {
        let (storage, _temp_dir) = setup_test_db();
        
        // Create nodes
        let node1 = storage.create_node("person", HashMap::new()).unwrap();
        let node2 = storage.create_node("person", HashMap::new()).unwrap();
        let node3 = storage.create_node("person", HashMap::new()).unwrap();
        
        // Create edges
        storage.create_edge("knows", &node1.id, &node2.id, HashMap::new()).unwrap();
        storage.create_edge("knows", &node1.id, &node3.id, HashMap::new()).unwrap();
        
        // Test out edges
        let out_edges = storage.get_out_edges(&node1.id, "knows").unwrap();
        assert_eq!(out_edges.len(), 2);
    }

    #[test]
    fn test_get_in_edges() {
        let (storage, _temp_dir) = setup_test_db();
        
        // Create nodes
        let node1 = storage.create_node("person", HashMap::new()).unwrap();
        let node2 = storage.create_node("person", HashMap::new()).unwrap();
        let node3 = storage.create_node("person", HashMap::new()).unwrap();
        
        // Create edges pointing to node1
        storage.create_edge("knows", &node2.id, &node1.id, HashMap::new()).unwrap();
        storage.create_edge("knows", &node3.id, &node1.id, HashMap::new()).unwrap();
        
        // Test in edges
        let in_edges = storage.get_in_edges(&node1.id, "knows").unwrap();
        assert_eq!(in_edges.len(), 2);
    }

    #[test]
    fn test_drop_node() {
        let (storage, _temp_dir) = setup_test_db();
        
        // Create nodes
        let node1 = storage.create_node("person", HashMap::new()).unwrap();
        let node2 = storage.create_node("person", HashMap::new()).unwrap();
        
        // Create edge
        storage.create_edge("knows", &node1.id, &node2.id, HashMap::new()).unwrap();
        
        // Drop node1
        storage.drop_node(&node1.id).unwrap();
        
        // Verify node1 is gone
        assert!(storage.get_node(&node1.id).is_err());
        
        // Verify node2 still exists
        assert!(storage.get_node(&node2.id).is_ok());
    }

    #[test]
    fn test_drop_edge() {
        let (storage, _temp_dir) = setup_test_db();
        
        // Create nodes
        let node1 = storage.create_node("person", HashMap::new()).unwrap();
        let node2 = storage.create_node("person", HashMap::new()).unwrap();
        
        // Create edge
        let edge = storage.create_edge(
            "knows",
            &node1.id,
            &node2.id,
            HashMap::new()
        ).unwrap();
        
        // Drop edge
        storage.drop_edge(&edge.id).unwrap();
        
        // Verify edge is gone
        assert!(storage.get_edge(&edge.id).is_err());
        
        // Verify nodes still exist
        assert!(storage.get_node(&node1.id).is_ok());
        assert!(storage.get_node(&node2.id).is_ok());
    }

    #[test]
    fn test_traversal_builder() {
        let (storage, _temp_dir) = setup_test_db();
        
        // Create test graph
        let node1 = storage.create_node("person", HashMap::new()).unwrap();
        let node2 = storage.create_node("person", HashMap::new()).unwrap();
        let node3 = storage.create_node("person", HashMap::new()).unwrap();
        
        storage.create_edge("knows", &node1.id, &node2.id, HashMap::new()).unwrap();
        storage.create_edge("knows", &node2.id, &node3.id, HashMap::new()).unwrap();
        
        // Test traversal
        let mut traversal = TraversalBuilder::new(vec![node1.clone()]);
        
        // Test out traversal
        traversal.out(&storage, "knows");
        if let TraversalValue::NodeArray(nodes) = &traversal.current_step[0] {
            assert_eq!(nodes.len(), 1);
            assert_eq!(nodes[0].id, node2.id);
        }
        
        // Test out_e traversal
        traversal = TraversalBuilder::new(vec![node1.clone()]);
        traversal.out_e(&storage, "knows");
        if let TraversalValue::EdgeArray(edges) = &traversal.current_step[0] {
            assert_eq!(edges.len(), 1);
            assert_eq!(edges[0].to_node, node2.id);
        }
    }

    #[test]
    fn test_check_exists() {
        let (storage, _temp_dir) = setup_test_db();
        
        let node = storage.create_node("person", HashMap::new()).unwrap();
        
        assert!(storage.check_exists(&node.id).unwrap());
        assert!(!storage.check_exists("non-existent-id").unwrap());
    }
}