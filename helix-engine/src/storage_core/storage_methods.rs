use core::fmt;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use crate::types::GraphError;
use protocol::{Node, Edge, Value};

pub trait StorageMethods {
    /// Checks whether an entry with a given id exists.
    /// Works for nodes or edges.
    fn check_exists(&self, id: &str) -> Result<bool, GraphError>;
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


    /// Gets a node object for a given node id
    fn get_node(&self, id: &str) -> Result<Node, GraphError>;
    /// Gets a edge object for a given edge id
    fn get_edge(&self, id: &str) -> Result<Edge, GraphError>;

    /// Returns a list of edge objects of the outgoing edges from a given node 
    fn get_out_edges(&self, node_id: &str, edge_label: &str) -> Result<Vec<Edge>, GraphError>;
    /// Returns a list of edge objects of the incoming edges from a given node
    fn get_in_edges(&self, node_id: &str, edge_label: &str) -> Result<Vec<Edge>, GraphError>;
    
    /// Returns a list of node objects of the outgoing nodes from a given node
    fn get_out_nodes(&self, node_id: &str, edge_label: &str) -> Result<Vec<Node>, GraphError>;
    /// Returns a list of node objects of the incoming nodes from a given node
    fn get_in_nodes(&self, node_id: &str, edge_label: &str) -> Result<Vec<Node>, GraphError>;

    /// Returns all nodes in the graph
    fn get_all_nodes(&self) -> Result<Vec<Node>, GraphError>;
    /// Returns all edges in the graph
    fn get_all_edges(&self) -> Result<Vec<Edge>, GraphError>;

    /// Creates a node entry
    fn create_node(
        &self,
        label: &str,
        properties: impl IntoIterator<Item = (String, Value)>,
    ) -> Result<Node, GraphError>;

    /// Creates an edge entry between two nodes
    fn create_edge(
        &self,
        label: &str,
        from_node: &str,
        to_node: &str,
        properties: impl IntoIterator<Item = (String, Value)>,
    ) -> Result<Edge, GraphError>;

    /// Deletes a node entry along with all of its connected edges 
    fn drop_node(&self, id: &str) -> Result<(), GraphError>;

    /// Deletes an edge entry
    fn drop_edge(&self, id: &str)  -> Result<(), GraphError>;
}
