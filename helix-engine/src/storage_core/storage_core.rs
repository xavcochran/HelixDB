use std::collections::HashMap;

use bincode::{deserialize, serialize};
use rocksdb::{IteratorMode, Options, WriteBatch, WriteBatchWithTransaction, DB};

use uuid::Uuid;

use crate::storage_core::graph::{Edge, GraphError, GraphMethods, Node, Value};

// Byte values of data-type key prefixes
const NODE_PREFIX: &[u8] = b"n:";
const EDGE_PREFIX: &[u8] = b"e:";
const NODE_LABEL_PREFIX: &[u8] = b"nl:";
const EDGE_LABEL_PREFIX: &[u8] = b"el:";
const OUT_EDGES_PREFIX: &[u8] = b"out:";
const IN_EDGES_PREFIX: &[u8] = b"in:";

pub struct HelixGraphStorage {
    db: DB,
}

// const path: &str = "./data/graph_data";

impl HelixGraphStorage {
    /// HelixGraphStorage struct constructor
    fn new(path: &str) -> Result<HelixGraphStorage, GraphError> {
        let mut opts = Options::default();
        opts.create_if_missing(true);
        let db = match DB::open(&opts, path) {
            Ok(db) => db,
            Err(err) => return Err(GraphError::from(err)),
        };
        Ok(Self { db })
    }

    /// Creates node key using the prefix and given id
    fn node_key(id: &str) -> Vec<u8> {
        [NODE_PREFIX, id.as_bytes()].concat()
    }

    /// Creates edge key using the prefix and given id
    fn edge_key(id: &str) -> Vec<u8> {
        [EDGE_PREFIX, id.as_bytes()].concat()
    }

    /// Creates node label key using the prefix, the given label, and id 
    fn node_label_key(label: &str, id: &str) -> Vec<u8> {
        [NODE_LABEL_PREFIX, label.as_bytes(), b":", id.as_bytes()].concat()
    }   

    /// Creates edge label key using the prefix, the given label, and  id 
    fn edge_label_key(label: &str, id: &str) -> Vec<u8> {
        [EDGE_LABEL_PREFIX, label.as_bytes(), b":", id.as_bytes()].concat()
    }

    /// Creates key for an outgoing edge using the prefix, source node id, and edge id
    fn out_edge_key(node_id: &str, edge_id: &str) -> Vec<u8> {
        [
            OUT_EDGES_PREFIX,
            node_id.as_bytes(),
            b":",
            edge_id.as_bytes(),
        ]
        .concat()
    }

    /// Creates key for an incoming edge using the prefix, sink node id, and edge id
    fn in_edge_key(node_id: &str, edge_id: &str) -> Vec<u8> {
        [
            IN_EDGES_PREFIX,
            node_id.as_bytes(),
            b":",
            edge_id.as_bytes(),
        ]
        .concat()
    }
}

impl GraphMethods for HelixGraphStorage {
    fn check_exists(&self, id: &str) -> Result<bool, GraphError> {
        match self.db.get_pinned(id) {
            Ok(Some(_)) => Ok(true),
            Ok(None) => Ok(false),
            Err(err) => Err(GraphError::from(err)),
        }
    }

    fn get_temp_node(&self, id: &str) -> Result<Node, GraphError> {
        match self.db.get_pinned(id) {
            Ok(Some(data)) => Ok(deserialize(&data).unwrap()),
            Ok(None) => Err(GraphError::Other(format!("Item not found!"))),
            Err(err) => Err(GraphError::from(err)),
        }
    }

    fn get_temp_edge(&self, id: &str) -> Result<Edge, GraphError> {
        match self.db.get_pinned(id) {
            Ok(Some(data)) => Ok(deserialize(&data).unwrap()),
            Ok(None) => Err(GraphError::Other(format!("Item not found!"))),
            Err(err) => Err(GraphError::from(err)),
        }
    }

    fn get_node(&self, id: &str) -> Result<Node, GraphError> {
        match self.db.get(Self::node_key(id)) {
            Ok(Some(data)) => Ok(deserialize(&data).unwrap()),
            Ok(None) => Err(GraphError::Other(format!("Item not found!"))),
            Err(err) => Err(GraphError::from(err)),
        }
    }
    fn get_edge(&self, id: &str) -> Result<Edge, GraphError> {
        match self.db.get(Self::edge_key(id)) {
            Ok(Some(data)) => Ok(deserialize(&data).unwrap()),
            Ok(None) => Err(GraphError::Other(format!("Item not found!"))),
            Err(err) => Err(GraphError::from(err)),
        }
    }

    fn create_node(
        &self,
        label: &str,
        properties: std::collections::HashMap<String, Value>,
    ) -> Result<Node, GraphError> {
        let node = Node {
            id: Uuid::new_v4().to_string(),
            label: label.to_string(),
            properties,
        };

        let mut new_batch = WriteBatchWithTransaction::default();

        new_batch.put(Self::node_key(&node.id), serialize(&node).unwrap());
        new_batch.put(Self::node_label_key(label, &node.id), vec![]);

        self.db.write(new_batch)?;
        Ok(node)
    }

    fn create_edge(
        &self,
        label: &str,
        from_node: &str,
        to_node: &str,
        properties: HashMap<String, Value>,
    ) -> Result<Edge, GraphError> {
        // look at creating check function that uses pinning
        if !self.get_node(from_node).is_ok() || !self.get_node(to_node).is_ok() {
            return Err(GraphError::Other(format!("One or both nodes do not exist")));
        }

        let edge = Edge {
            id: Uuid::new_v4().to_string(),
            label: label.to_string(),
            from_node: from_node.to_string(),
            to_node: to_node.to_string(),
            properties,
        };

        let mut batch = WriteBatch::default();

        // new edge
        batch.put(Self::edge_key(&edge.id), bincode::serialize(&edge).unwrap());
        // edge label
        batch.put(Self::edge_label_key(label, &edge.id), vec![]);

        // edge keys
        batch.put(Self::out_edge_key(from_node, &edge.id), vec![]);
        batch.put(Self::in_edge_key(to_node, &edge.id), vec![]);

        self.db.write(batch)?;
        Ok(edge)
    }

    fn drop_node(&self, id: &str) -> Result<(), GraphError> {
        // get out edges
        let out_prefix = Self::out_edge_key(id, "");
        let iter = self
            .db
            .iterator(IteratorMode::From(&out_prefix, rocksdb::Direction::Forward));
        // delete them
        for result in iter {
            let (key, _) = result?;
            if !key.starts_with(&out_prefix) {
                break;
            }

            let edge_id = String::from_utf8(key[out_prefix.len()..].to_vec()).unwrap();
            self.drop_edge(&edge_id)?;
        }

        // get in edges
        let in_prefix = Self::in_edge_key(id, "");
        let iter = self
            .db
            .iterator(IteratorMode::From(&in_prefix, rocksdb::Direction::Forward));
        // delete them
        for result in iter {
            let (key, _) = result?;
            if !key.starts_with(&in_prefix) {
                break;
            }

            let edge_id = String::from_utf8(key[in_prefix.len()..].to_vec()).unwrap();
            self.drop_edge(&edge_id)?;
        }

        // delete node
        self.db.delete(Self::node_key(id))?;

        Ok(())
    }

    fn drop_edge(&self, id: &str) -> Result<(), GraphError> {
        match self.db.delete(Self::edge_key(id)) {
            Ok(_) => Ok(()),
            Err(err) => Err(GraphError::from(err)),
        }
    }
}
