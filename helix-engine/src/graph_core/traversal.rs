use crate::types::{Node, Edge, Value, GraphError};
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

pub struct Traversal<'a> {
    pub variables: HashMap<String, TraversalValue<'a>>
}


