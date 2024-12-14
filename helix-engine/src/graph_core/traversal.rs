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
                    let node_obj = storage.get_node(&edge.from_node).unwrap();
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
    use tempfile::TempDir;
    use std::collections::HashMap;

    fn setup_test_db() -> (HelixGraphStorage, TempDir) {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().to_str().unwrap();
        let storage = HelixGraphStorage::new(db_path).unwrap();
        (storage, temp_dir)
    }

    #[test]
    fn test_add_v() {
        let (storage, _temp_dir) = setup_test_db();
        let mut traversal = TraversalBuilder::new(vec![]);
        
        traversal.add_v(&storage, "person");
        
        match &traversal.current_step[0] {
            TraversalValue::SingleNode(node) => {
                assert_eq!(node.label, "person");
            }
            _ => panic!("Expected SingleNode value"),
        }
    }

    #[test]
    fn test_add_e() {
        let (storage, _temp_dir) = setup_test_db();
        
        let node1 = storage.create_node("person", HashMap::new()).unwrap();
        let node2 = storage.create_node("person", HashMap::new()).unwrap();
        
        let mut traversal = TraversalBuilder::new(vec![]);
        traversal.add_e(&storage, "knows", &node1.id, &node2.id);
        
        match &traversal.current_step[0] {
            TraversalValue::SingleEdge(edge) => {
                assert_eq!(edge.label, "knows");
                assert_eq!(edge.from_node, node1.id);
                assert_eq!(edge.to_node, node2.id);
            }
            _ => panic!("Expected SingleEdge value"),
        }
    }

    #[test]
    fn test_out() {
        let (storage, _temp_dir) = setup_test_db();
        
        // Create graph: (person1)-[knows]->(person2)-[knows]->(person3)
        let person1 = storage.create_node("person", HashMap::new()).unwrap();
        let person2 = storage.create_node("person", HashMap::new()).unwrap();
        let person3 = storage.create_node("person", HashMap::new()).unwrap();
        
        storage.create_edge("knows", &person1.id, &person2.id, HashMap::new()).unwrap();
        storage.create_edge("knows", &person2.id, &person3.id, HashMap::new()).unwrap();
        
        let mut traversal = TraversalBuilder::new(vec![person1.clone()]);
        traversal.out(&storage, "knows");
        
        match &traversal.current_step[0] {
            TraversalValue::NodeArray(nodes) => {
                assert_eq!(nodes.len(), 1);
                assert_eq!(nodes[0].id, person2.id);
            }
            _ => panic!("Expected NodeArray value"),
        }
    }

    #[test]
    fn test_out_e() {
        let (storage, _temp_dir) = setup_test_db();
        
        // Create graph: (person1)-[knows]->(person2)
        let person1 = storage.create_node("person", HashMap::new()).unwrap();
        let person2 = storage.create_node("person", HashMap::new()).unwrap();
        
        let edge = storage.create_edge("knows", &person1.id, &person2.id, HashMap::new()).unwrap();
        
        let mut traversal = TraversalBuilder::new(vec![person1.clone()]);
        traversal.out_e(&storage, "knows");
        
        match &traversal.current_step[0] {
            TraversalValue::EdgeArray(edges) => {
                assert_eq!(edges.len(), 1);
                assert_eq!(edges[0].id, edge.id);
                assert_eq!(edges[0].label, "knows");
            }
            _ => panic!("Expected EdgeArray value"),
        }
    }

    #[test]
    fn test_in() {
        let (storage, _temp_dir) = setup_test_db();
        
        // Create graph: (person1)-[knows]->(person2)
        let person1 = storage.create_node("person", HashMap::new()).unwrap();
        let person2 = storage.create_node("person", HashMap::new()).unwrap();
        
        storage.create_edge("knows", &person1.id, &person2.id, HashMap::new()).unwrap();
        
        let mut traversal = TraversalBuilder::new(vec![person2.clone()]);
        traversal.in_(&storage, "knows");
        
        match &traversal.current_step[0] {
            TraversalValue::NodeArray(nodes) => {
                assert_eq!(nodes.len(), 1);
                assert_eq!(nodes[0].id, person1.id );
            }
            _ => panic!("Expected NodeArray value"),
        }
    }

    #[test]
    fn test_in_e() {
        let (storage, _temp_dir) = setup_test_db();
        
        // Create test graph: (person1)-[knows]->(person2)
        let person1 = storage.create_node("person", HashMap::new()).unwrap();
        let person2 = storage.create_node("person", HashMap::new()).unwrap();
        
        let edge = storage.create_edge("knows", &person1.id, &person2.id, HashMap::new()).unwrap();
        
        let mut traversal = TraversalBuilder::new(vec![person2.clone()]);
        traversal.in_e(&storage, "knows");
        
        match &traversal.current_step[0] {
            TraversalValue::EdgeArray(edges) => {
                assert_eq!(edges.len(), 1);
                assert_eq!(edges[0].id, edge.id);
                assert_eq!(edges[0].label, "knows");
            }
            _ => panic!("Expected EdgeArray value"),
        }
    }

    #[test]
    fn test_traversal_validation() {
        let (storage, _temp_dir) = setup_test_db();
        let mut traversal = TraversalBuilder::new(vec![]);
        
        let node1 = storage.create_node("person", HashMap::new()).unwrap();
        let node2 = storage.create_node("person", HashMap::new()).unwrap();
        let edge = storage.create_edge("knows", &node1.id, &node2.id, HashMap::new()).unwrap();
        traversal.current_step = vec![TraversalValue::SingleEdge(edge)];
        
        assert!(traversal.check_is_valid_node_traversal("test").is_err());
        
        traversal.current_step = vec![TraversalValue::SingleNode(node1)];
        assert!(traversal.check_is_valid_edge_traversal("test").is_err());
    }

    #[test]
    fn test_complex_traversal() {
        let (storage, _temp_dir) = setup_test_db();
        
        // Graph structure:
        // (person1)-[knows]->(person2)-[likes]->(person3)
        //     ^                                    |
        //     |                                    |
        //     +--------------------[follows]-------+
        
        let person1 = storage.create_node("person", HashMap::new()).unwrap();
        let person2 = storage.create_node("person", HashMap::new()).unwrap();
        let person3 = storage.create_node("person", HashMap::new()).unwrap();
        
        storage.create_edge("knows", &person1.id, &person2.id, HashMap::new()).unwrap();
        storage.create_edge("likes", &person2.id, &person3.id, HashMap::new()).unwrap();
        storage.create_edge("follows", &person3.id, &person1.id, HashMap::new()).unwrap();
        
        let mut traversal = TraversalBuilder::new(vec![person1.clone()]);
        
        traversal.out(&storage, "knows");
        
        match &traversal.current_step[0] {
            TraversalValue::NodeArray(nodes) => {
                assert_eq!(nodes.len(), 1);
                assert_eq!(nodes[0].id, person2.id);
            }
            _ => panic!("Expected NodeArray value"),
        }
        
        traversal.out(&storage, "likes");
        
        match &traversal.current_step[0] {
            TraversalValue::NodeArray(nodes) => {
                assert_eq!(nodes.len(), 1);
                assert_eq!(nodes[0].id, person3.id);
            }
            _ => panic!("Expected NodeArray value"),
        }
    }
}