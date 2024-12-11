use rocksdb::{IteratorMode, Options, WriteBatch, DB};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

use crate::storage_core::graph::{GraphMethods};

#[derive(Serialize, Deserialize)]
pub struct Node {
    id: String,
    label: String,
    properties: HashMap<String, Value>,
}

#[derive(Serialize, Deserialize)]
pub struct Edge {
    id: String,
    label: String,
    from_node: String,
    to_node: String,
    properties: HashMap<String, Value>,
}

#[derive(Serialize, Deserialize)]
pub enum Value {
    String(String),
    Number(f64),
    Boolean(bool),
    Array(Vec<Value>),
    Null,
}

// data type key prefixes as bytes
const NODE_PREFIX: &[u8] = b"n:";
const EDGE_PREFIX: &[u8] = b"e:";
const NODE_LABEL_PREFIX: &[u8] = b"nl:";
const EDGE_LABEL_PREFIX: &[u8] = b"el:";
const OUT_EDGES_PREFIX: &[u8] = b"out:";
const IN_EDGES_PREFIX: &[u8] = b"in:";

pub struct HelixGraph {
    db: DB,
}

// const path: &str = "./data/graph_data";

// constructor
impl HelixGraph {
    fn new(path: &str) -> Result<HelixGraph, rocksdb::Error> {
        let mut opts = Options::default();
        opts.create_if_missing(true);
        let db = match DB::open(&opts, path) {
            Ok(db) => db,
            Err(err) => return Err(err),
        };
        Ok(Self { db })
    }
}
