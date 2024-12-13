use crate::{
    storage_core::{storage_core::HelixGraphStorage, storage_methods::StorageMethods},
    types::{Edge, GraphError, Node, Value},
};
use std::collections::HashMap;

#[derive(Debug)]
pub enum TraversalValue<'a> {
    SingleNode(&'a Node),
    SingleEdge(&'a Edge),
    SingleValue(&'a Value),
    NodeArray(Vec<&'a Node>),
    EdgeArray(Vec<&'a Edge>),
    ValueArray(Vec<&'a Value>),
}

pub struct TraversalBuilder<'a> {
    variables: HashMap<String, TraversalValue<'a>>,
    current_step: Vec<TraversalValue<'a>>,
}

impl<'a> TraversalBuilder<'a> {
    pub fn new(start_nodes: Vec<&'a Node>) -> Self {
        let mut builder = Self {
            variables: HashMap::new(),
            current_step: vec![TraversalValue::NodeArray(start_nodes)],
        };
        builder
    }

    pub fn is_valid_node_traversal(&self) -> bool {}

    pub fn out(&mut self, storage: &HelixGraphStorage, edge_label: &str) -> &mut Self {
        if let TraversalValue::NodeArray(nodes) = self.current_step[0] {
            let mut new_current: Vec<TraversalValue::NodeArray> = Vec::with_capacity(nodes.len());
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

    pub fn out_e(&mut self, storage: &HelixGraphStorage, edge_label: &str) -> &mut Self {
        if let TraversalValue::NodeArray(nodes) = self.current_step[0] {
            let mut new_current: Vec<TraversalValue::EdgeArray> = Vec::with_capacity(nodes.len());
            for node in nodes {
                let edges = storage.get_out_edges(&node.id, edge_label).unwrap();
                new_current.push(TraversalValue::EdgeArray(edges));
            }
            self.current_step = new_current;
        }
        self
    }
}

// need to account for multiple nodes or edges at a given traversal step
//
