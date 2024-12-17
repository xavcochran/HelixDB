use crate::{
    props,
    storage_core::{storage_core::HelixGraphStorage, storage_methods::StorageMethods},
    types::GraphError,
};
use protocol::{Edge, Node, Value};
use rocksdb::properties;
use std::collections::HashMap;
use std::time::Instant;

use super::traversal_steps::{SourceTraversalSteps, TraversalSteps};

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
            variables: HashMap::from_iter(props!()),
            current_step: vec![TraversalValue::NodeArray(start_nodes)],
        };
        builder
    }

    pub fn check_is_valid_node_traversal(&self, function_name: &str) -> Result<(), GraphError> {
        match matches!(
            self.current_step[0],
            TraversalValue::NodeArray(_) | TraversalValue::SingleNode(_)
        ) {
            true => Ok(()),
            false => Err(GraphError::TraversalError(format!(
                "The traversal step {:?}, is not a valid traversal from an edge. 
                The current step should be a node.",
                function_name
            ))),
        }
    }

    pub fn check_is_valid_edge_traversal(&self, function_name: &str) -> Result<(), GraphError> {
        match matches!(
            self.current_step[0],
            TraversalValue::EdgeArray(_) | TraversalValue::SingleEdge(_)
        ) {
            true => Ok(()),
            false => Err(GraphError::TraversalError(format!(
                "The traversal step {:?}, is not a valid traversal from a node. 
                The current step should be an edge",
                function_name
            ))),
        }
    }
}

impl SourceTraversalSteps for TraversalBuilder {
    fn v(&mut self, storage: &HelixGraphStorage) -> &mut Self {
        let nodes = storage.get_all_nodes().unwrap(); // TODO: Handle error
        self.current_step = vec![TraversalValue::NodeArray(nodes)];
        self
    }

    fn e(&mut self, storage: &HelixGraphStorage) -> &mut Self {
        let edges = storage.get_all_edges().unwrap(); // TODO: Handle error
        self.current_step = vec![TraversalValue::EdgeArray(edges)];
        self
    }

    fn add_v(&mut self, storage: &HelixGraphStorage, node_label: &str) -> &mut Self {
        let node = storage.create_node(node_label, props!()).unwrap(); // TODO: Handle error
        self.current_step = vec![TraversalValue::SingleNode(node)];
        self
    }

    fn add_e(
        &mut self,
        storage: &HelixGraphStorage,
        edge_label: &str,
        from_id: &str,
        to_id: &str,
    ) -> &mut Self {
        let edge = storage
            .create_edge(edge_label, from_id, to_id, props!())
            .unwrap(); // TODO: Handle error
        self.current_step = vec![TraversalValue::SingleEdge(edge)];
        self
    }
}

impl TraversalSteps for TraversalBuilder {

    fn out(&mut self, storage: &HelixGraphStorage, edge_label: &str) -> &mut Self {
        self.check_is_valid_node_traversal("out")
            .unwrap(); // TODO: Handle error

        if let TraversalValue::NodeArray(nodes) = &self.current_step[0] {
            let mut new_current = Vec::with_capacity(nodes.len());
            for node in nodes {
                new_current.push(TraversalValue::NodeArray(
                    storage.get_out_nodes(&node.id, edge_label).unwrap(), // TODO: Handle error
                ));
            }
            self.current_step = new_current;
        }
        self
    }

    fn out_e(&mut self, storage: &HelixGraphStorage, edge_label: &str) -> &mut Self {
        self.check_is_valid_node_traversal("out_e")
            .unwrap(); // TODO: Handle error
        if let TraversalValue::NodeArray(nodes) = &self.current_step[0] {
            let mut new_current: Vec<TraversalValue> = Vec::with_capacity(nodes.len());
            for node in nodes {
                new_current.push(TraversalValue::EdgeArray(
                    storage.get_out_edges(&node.id, edge_label).unwrap(), // TODO: Handle error
                ));
            }
            self.current_step = new_current;
        }
        self
    }


    fn in_(&mut self, storage: &HelixGraphStorage, edge_label: &str) -> &mut Self {
        self.check_is_valid_node_traversal("in_")
            .unwrap();
        if let TraversalValue::NodeArray(nodes) = &self.current_step[0] {
            let mut new_current: Vec<TraversalValue> = Vec::with_capacity(nodes.len());
            for node in nodes {
                new_current.push(TraversalValue::NodeArray(
                    storage.get_in_nodes(&node.id, edge_label).unwrap(), // TODO: Handle error
                ));
            }
            self.current_step = new_current;
        }
        self
    }


    fn in_e(&mut self, storage: &HelixGraphStorage, edge_label: &str) -> &mut Self {
        self.check_is_valid_node_traversal("in_e")
            .unwrap();
        if let TraversalValue::NodeArray(nodes) = &self.current_step[0] {
            let mut new_current: Vec<TraversalValue> = Vec::with_capacity(nodes.len());
            for node in nodes {
                new_current.push(TraversalValue::EdgeArray(
                    storage.get_in_edges(&node.id, edge_label).unwrap(), // TODO: Handle error
                ));
            }
            self.current_step = new_current;
        }
        self
    }
}

#[cfg(test)]
mod tests {
    use crate::props;

    use super::*;
    use rand::{random, Rng};
    use std::collections::HashMap;
    use tempfile::TempDir;

    fn setup_test_db() -> (HelixGraphStorage, TempDir) {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().to_str().unwrap();
        let storage = HelixGraphStorage::new(db_path).unwrap();
        (storage, temp_dir)
    }

    #[test]
    fn test_v() {
        let (storage, _temp_dir) = setup_test_db();

        let person1 = storage.create_node("person", props!()).unwrap();
        let person2 = storage.create_node("person", props!()).unwrap();
        let thing = storage.create_node("thing", props!()).unwrap();

        let mut traversal = TraversalBuilder::new(vec![]);
        traversal.v(&storage);

        // Check that the node array contains all nodes
        match &traversal.current_step[0] {
            TraversalValue::NodeArray(nodes) => {
                assert_eq!(nodes.len(), 3);

                let node_ids: Vec<String> = nodes.iter().map(|n| n.id.clone()).collect();
                let node_labels: Vec<String> = nodes.iter().map(|n| n.label.clone()).collect();

                assert!(node_ids.contains(&person1.id));
                assert!(node_ids.contains(&person2.id));
                assert!(node_ids.contains(&thing.id));

                assert_eq!(node_labels.iter().filter(|&l| l == "person").count(), 2);
                assert_eq!(node_labels.iter().filter(|&l| l == "thing").count(), 1);
            }
            _ => panic!("Expected NodeArray value"),
        }
    }

    #[test]
    fn test_e() {
        let (storage, _temp_dir) = setup_test_db();

        // Graph Structure:
        // (person1)-[knows]->(person2)
        //         \-[likes]->(person3)
        // (person2)-[follows]->(person3)

        let person1 = storage.create_node("person", props!()).unwrap();
        let person2 = storage.create_node("person", props!()).unwrap();
        let person3 = storage.create_node("person", props!()).unwrap();

        let knows_edge = storage
            .create_edge("knows", &person1.id, &person2.id, props!())
            .unwrap();
        let likes_edge = storage
            .create_edge("likes", &person1.id, &person3.id, props!())
            .unwrap();
        let follows_edge = storage
            .create_edge("follows", &person2.id, &person3.id, props!())
            .unwrap();

        let mut traversal = TraversalBuilder::new(vec![]);
        traversal.e(&storage);

        // Check that the edge array contains the three edges
        match &traversal.current_step[0] {
            TraversalValue::EdgeArray(edges) => {
                assert_eq!(edges.len(), 3);

                let edge_ids: Vec<String> = edges.iter().map(|e| e.id.clone()).collect();
                let edge_labels: Vec<String> = edges.iter().map(|e| e.label.clone()).collect();

                assert!(edge_ids.contains(&knows_edge.id));
                assert!(edge_ids.contains(&likes_edge.id));
                assert!(edge_ids.contains(&follows_edge.id));

                assert!(edge_labels.contains(&"knows".to_string()));
                assert!(edge_labels.contains(&"likes".to_string()));
                assert!(edge_labels.contains(&"follows".to_string()));

                for edge in edges {
                    match edge.label.as_str() {
                        "knows" => {
                            assert_eq!(edge.from_node, person1.id);
                            assert_eq!(edge.to_node, person2.id);
                        }
                        "likes" => {
                            assert_eq!(edge.from_node, person1.id);
                            assert_eq!(edge.to_node, person3.id);
                        }
                        "follows" => {
                            assert_eq!(edge.from_node, person2.id);
                            assert_eq!(edge.to_node, person3.id);
                        }
                        _ => panic!("Unexpected edge label"),
                    }
                }
            }
            _ => panic!("Expected EdgeArray value"),
        }
    }

    #[test]
    fn test_v_empty_graph() {
        let (storage, _temp_dir) = setup_test_db();

        let mut traversal = TraversalBuilder::new(vec![]);
        traversal.v(&storage);

        // Check that the node array is empty
        match &traversal.current_step[0] {
            TraversalValue::NodeArray(nodes) => {
                assert_eq!(nodes.len(), 0);
            }
            _ => panic!("Expected NodeArray value"),
        }
    }

    #[test]
    fn test_e_empty_graph() {
        let (storage, _temp_dir) = setup_test_db();

        let mut traversal = TraversalBuilder::new(vec![]);
        traversal.e(&storage);

        // Check that the edge array is empty
        match &traversal.current_step[0] {
            TraversalValue::EdgeArray(edges) => {
                assert_eq!(edges.len(), 0);
            }
            _ => panic!("Expected EdgeArray value"),
        }
    }

    #[test]
    fn test_v_nodes_without_edges() {
        let (storage, _temp_dir) = setup_test_db();

        let person1 = storage.create_node("person", props!()).unwrap();
        let person2 = storage.create_node("person", props!()).unwrap();

        let mut traversal = TraversalBuilder::new(vec![]);
        traversal.v(&storage);

        // Check that the node array contains the two nodes
        match &traversal.current_step[0] {
            TraversalValue::NodeArray(nodes) => {
                assert_eq!(nodes.len(), 2);
                let node_ids: Vec<String> = nodes.iter().map(|n| n.id.clone()).collect();
                assert!(node_ids.contains(&person1.id));
                assert!(node_ids.contains(&person2.id));
            }
            _ => panic!("Expected NodeArray value"),
        }
    }

    #[test]
    fn test_add_v() {
        let (storage, _temp_dir) = setup_test_db();
        let mut traversal = TraversalBuilder::new(vec![]);

        traversal.add_v(&storage, "person");

        // Check that the current step contains a single node
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

        let node1 = storage.create_node("person", props!()).unwrap();
        let node2 = storage.create_node("person", props!()).unwrap();

        let mut traversal = TraversalBuilder::new(vec![]);
        traversal.add_e(&storage, "knows", &node1.id, &node2.id);

        // Check that the current step contains a single edge
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
        let person1 = storage.create_node("person", props!()).unwrap();
        let person2 = storage.create_node("person", props!()).unwrap();
        let person3 = storage.create_node("person", props!()).unwrap();

        storage
            .create_edge("knows", &person1.id, &person2.id, props!())
            .unwrap();
        storage
            .create_edge("knows", &person2.id, &person3.id, props!())
            .unwrap();

        let mut traversal = TraversalBuilder::new(vec![person1.clone()]);
        // Traverse from person1 to person2
        traversal.out(&storage, "knows");

        // Check that current step is at person2
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
        let person1 = storage.create_node("person", props!()).unwrap();
        let person2 = storage.create_node("person", props!()).unwrap();

        let edge = storage
            .create_edge("knows", &person1.id, &person2.id, props!())
            .unwrap();

        let mut traversal = TraversalBuilder::new(vec![person1.clone()]);
        // Traverse from person1 to person2
        traversal.out_e(&storage, "knows");

        // Check that current step is at the edge between person1 and person2
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
        let person1 = storage.create_node("person", props!()).unwrap();
        let person2 = storage.create_node("person", props!()).unwrap();

        storage
            .create_edge("knows", &person1.id, &person2.id, props!())
            .unwrap();

        let mut traversal = TraversalBuilder::new(vec![person2.clone()]);

        // Traverse from person2 to person1
        traversal.in_(&storage, "knows");

        // Check that current step is at person1
        match &traversal.current_step[0] {
            TraversalValue::NodeArray(nodes) => {
                assert_eq!(nodes.len(), 1);
                assert_eq!(nodes[0].id, person1.id);
            }
            _ => panic!("Expected NodeArray value"),
        }
    }

    #[test]
    fn test_in_e() {
        let (storage, _temp_dir) = setup_test_db();

        // Create test graph: (person1)-[knows]->(person2)
        let person1 = storage.create_node("person", props!()).unwrap();
        let person2 = storage.create_node("person", props!()).unwrap();

        let edge = storage
            .create_edge("knows", &person1.id, &person2.id, props!())
            .unwrap();

        let mut traversal = TraversalBuilder::new(vec![person2.clone()]);
        // Traverse from person2 to person1
        traversal.in_e(&storage, "knows");

        // Check that current step is at the edge between person1 and person2
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

        let node1 = storage.create_node("person", props!()).unwrap();
        let node2 = storage.create_node("person", props!()).unwrap();
        let edge = storage
            .create_edge("knows", &node1.id, &node2.id, props!())
            .unwrap();
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
        //     ^                                     |
        //     |                                     |
        //     +-------<------[follows]------<-------+

        let person1 = storage.create_node("person", props!()).unwrap();
        let person2 = storage.create_node("person", props!()).unwrap();
        let person3 = storage.create_node("person", props!()).unwrap();

        storage
            .create_edge("knows", &person1.id, &person2.id, props!())
            .unwrap();
        storage
            .create_edge("likes", &person2.id, &person3.id, props!())
            .unwrap();
        storage
            .create_edge("follows", &person3.id, &person1.id, props!())
            .unwrap();

        let mut traversal = TraversalBuilder::new(vec![person1.clone()]);

        // Traverse from person1 to person2
        traversal.out(&storage, "knows");

        // Check that current step is at person2
        match &traversal.current_step[0] {
            TraversalValue::NodeArray(nodes) => {
                assert_eq!(nodes.len(), 1);
                assert_eq!(nodes[0].id, person2.id);
            }
            _ => panic!("Expected NodeArray value"),
        }

        // Traverse from person2 to person3
        traversal.out(&storage, "likes");

        // Check that current step is at person3
        match &traversal.current_step[0] {
            TraversalValue::NodeArray(nodes) => {
                assert_eq!(nodes.len(), 1);
                assert_eq!(nodes[0].id, person3.id);
            }
            _ => panic!("Expected NodeArray value"),
        }

        // Traverse from person3 to person1
        traversal.out(&storage, "follows");

        // Check that current step is at person1
        match &traversal.current_step[0] {
            TraversalValue::NodeArray(nodes) => {
                assert_eq!(nodes.len(), 1);
                assert_eq!(nodes[0].id, person1.id);
            }
            _ => panic!("Expected NodeArray value"),
        }
    }
}
