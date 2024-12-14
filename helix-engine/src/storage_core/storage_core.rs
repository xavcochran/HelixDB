use std::collections::HashMap;

use bincode::{deserialize, serialize};
use rocksdb::{IteratorMode, Options, WriteBatch, WriteBatchWithTransaction, DB};

use uuid::Uuid;

use crate::storage_core::storage_methods::StorageMethods;
use crate::types::{Edge, GraphError, Node, Value};

// Byte values of data-type key prefixes
const NODE_PREFIX: &[u8] = b"n:";
const EDGE_PREFIX: &[u8] = b"e:";
const NODE_LABEL_PREFIX: &[u8] = b"nl:";
const EDGE_LABEL_PREFIX: &[u8] = b"el:";
const OUT_EDGES_PREFIX: &[u8] = b"o:";
const IN_EDGES_PREFIX: &[u8] = b"i:";

pub struct HelixGraphStorage {
    db: DB,
}

// const path: &str = "./data/graph_data";

impl HelixGraphStorage {
    /// HelixGraphStorage struct constructor
    pub fn new(path: &str) -> Result<HelixGraphStorage, GraphError> {
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
    /// 35 Bytes
    fn out_edge_key(source_node_id: &str, edge_id: &str) -> Vec<u8> {
        [
            OUT_EDGES_PREFIX,
            source_node_id.as_bytes(),
            b":",
            edge_id.as_bytes(),
        ]
        .concat()
    }

    /// Creates key for an incoming edge using the prefix, sink node id, and edge id
    /// 35 Bytes
    fn in_edge_key(sink_node_id: &str, edge_id: &str) -> Vec<u8> {
        [
            OUT_EDGES_PREFIX,
            sink_node_id.as_bytes(),
            b":",
            edge_id.as_bytes(),
        ]
        .concat()
    }
}

impl StorageMethods for HelixGraphStorage {
    fn check_exists(&self, id: &str) -> Result<bool, GraphError> {
        match self.db.get_pinned(Self::node_key(id)) {
            Ok(Some(_)) => Ok(true),
            Ok(None) => Ok(false),
            Err(err) => Err(GraphError::from(err)),
        }
    }

    fn get_temp_node(&self, id: &str) -> Result<Node, GraphError> {
        match self.db.get_pinned(Self::node_key(id)) {
            Ok(Some(data)) => Ok(deserialize(&data).unwrap()),
            Ok(None) => Err(GraphError::New(format!("Item not found!"))),
            Err(err) => Err(GraphError::from(err)),
        }
    }

    fn get_temp_edge(&self, id: &str) -> Result<Edge, GraphError> {
        match self.db.get_pinned(Self::edge_key(id)) {
            Ok(Some(data)) => Ok(deserialize(&data).unwrap()),
            Ok(None) => Err(GraphError::New(format!("Item not found!"))),
            Err(err) => Err(GraphError::from(err)),
        }
    }

    fn get_node(&self, id: &str) -> Result<Node, GraphError> {
        match self.db.get(Self::node_key(id)) {
            Ok(Some(data)) => Ok(deserialize(&data).unwrap()),
            Ok(None) => Err(GraphError::New(format!("Item not found!"))),
            Err(err) => Err(GraphError::from(err)),
        }
    }
    fn get_edge(&self, id: &str) -> Result<Edge, GraphError> {
        match self.db.get([EDGE_PREFIX, id.as_bytes()].concat()) {
            Ok(Some(data)) => Ok(deserialize(&data).unwrap()),
            Ok(None) => Err(GraphError::New(format!("Item not found!"))),
            Err(err) => Err(GraphError::from(err)),
        }
    }

    fn get_out_edges(&self, node_id: &str, edge_label: &str) -> Result<Vec<Edge>, GraphError> {
        let mut edges = Vec::new();
        // get out edges
        let out_prefix = Self::out_edge_key(node_id, "");
        let iter = self
            .db
            .iterator(IteratorMode::From(&out_prefix, rocksdb::Direction::Forward));

        // get edge values
        for result in iter {
            let (key, _) = result?;
            if !key.starts_with(&out_prefix) {
                break;
            }

            let edge_id = String::from_utf8(key[out_prefix.len()..].to_vec()).unwrap();
            let edge = self.get_edge(&edge_id).unwrap();
            if edge.label.as_str() == edge_label {
                edges.push(edge);
            }
        }
        Ok(edges)
    }

    fn get_in_edges(&self, node_id: &str, edge_label: &str) -> Result<Vec<Edge>, GraphError> {
        let mut edges = Vec::new();
        // get in edges
        let in_prefix = Self::in_edge_key(node_id, "");
        let iter = self
            .db
            .iterator(IteratorMode::From(&in_prefix, rocksdb::Direction::Forward));

        // get edge values
        for result in iter {
            let (key, _) = result?;
            if !key.starts_with(&in_prefix) {
                break;
            }

            let edge_id = String::from_utf8(key[in_prefix.len()..].to_vec()).unwrap();
            let edge = self.get_edge(&edge_id).unwrap();
            if edge.label.as_str() == edge_label {
                edges.push(edge);
            }
        }
        Ok(edges)
    }

    fn get_all_nodes(&self) -> Result<Vec<Node>, GraphError> {
        let node_prefix = Self::node_key("");
        let mut nodes = Vec::new();
        
        let iter = self.db.iterator(IteratorMode::From(
            &node_prefix,
            rocksdb::Direction::Forward,
        ));
    
        for result in iter {
            let (key, value) = result?;
            if !key.starts_with(&node_prefix) {
                break;
            }
            nodes.push(deserialize(&value).unwrap());
        }
        
        Ok(nodes)
    }

    fn get_all_edges(&self) -> Result<Vec<Edge>, GraphError> {
        let edge_prefix = Self::edge_key("");
        let mut edges = Vec::new();
        
        let iter = self.db.iterator(IteratorMode::From(
            &edge_prefix,
            rocksdb::Direction::Forward,
        ));
    
        for result in iter {
            let (key, value) = result?;
            if !key.starts_with(&edge_prefix) {
                break;
            }
            edges.push(deserialize(&value).unwrap());
        }
        
        Ok(edges)
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
            return Err(GraphError::New(format!("One or both nodes do not exist")));
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

    fn drop_edge(&self, edge_id: &str) -> Result<(), GraphError> {
        let edge_data = self.db.get_pinned(Self::edge_key(edge_id))?.unwrap();
        let edge: Edge = deserialize(&edge_data).unwrap();

        let mut batch = WriteBatch::default();

        batch.delete(Self::out_edge_key(&edge.from_node, edge_id));
        batch.delete(Self::in_edge_key(&edge.to_node, edge_id));
        batch.delete(Self::edge_key(edge_id));

        match self.db.write(batch) {
            Ok(_) => Ok(()),
            Err(err) => Err(GraphError::from(err)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage_core::storage_methods::StorageMethods;
    use crate::types::{Edge, GraphError, Node, Value};
    use std::collections::HashMap;
    use tempfile::TempDir;

    fn setup_temp_db() -> (HelixGraphStorage, TempDir) {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().to_str().unwrap();
        let storage = HelixGraphStorage::new(db_path).unwrap();
        (storage, temp_dir)
    }

    #[test]
    fn test_create_node() {
        let (storage, _temp_dir) = setup_temp_db();
        let mut properties = HashMap::new();
        properties.insert("name".to_string(), Value::String("test node".to_string()));

        let node = storage.create_node("person", properties).unwrap();

        let retrieved_node = storage.get_node(&node.id).unwrap();
        assert_eq!(node.id, retrieved_node.id);
        assert_eq!(node.label, "person");
        assert_eq!(
            node.properties.get("name").unwrap(),
            &Value::String("test node".to_string())
        );
    }

    #[test]
    fn test_create_edge() {
        let (storage, _temp_dir) = setup_temp_db();

        let node1 = storage.create_node("person", HashMap::new()).unwrap();
        let node2 = storage.create_node("person", HashMap::new()).unwrap();

        let mut edge_props = HashMap::new();
        edge_props.insert("weight".to_string(), Value::Integer(5));

        let edge = storage
            .create_edge("knows", &node1.id, &node2.id, edge_props)
            .unwrap();

        let retrieved_edge = storage.get_edge(&edge.id).unwrap();
        assert_eq!(edge.id, retrieved_edge.id);
        assert_eq!(edge.label, "knows");
        assert_eq!(edge.from_node, node1.id);
        assert_eq!(edge.to_node, node2.id);
    }

    #[test]
    fn test_create_edge_with_nonexistent_nodes() {
        let (storage, _temp_dir) = setup_temp_db();

        let result = storage.create_edge("knows", "nonexistent1", "nonexistent2", HashMap::new());

        assert!(result.is_err());
    }

    #[test]
    fn test_drop_node() {
        let (storage, _temp_dir) = setup_temp_db();

        let node1 = storage.create_node("person", HashMap::new()).unwrap();
        let node2 = storage.create_node("person", HashMap::new()).unwrap();
        let node3 = storage.create_node("person", HashMap::new()).unwrap();

        storage
            .create_edge("knows", &node1.id, &node2.id, HashMap::new())
            .unwrap();
        storage
            .create_edge("knows", &node3.id, &node1.id, HashMap::new())
            .unwrap();

        storage.drop_node(&node1.id).unwrap();

        assert!(storage.get_node(&node1.id).is_err());
    }

    #[test]
    fn test_drop_edge() {
        let (storage, _temp_dir) = setup_temp_db();

        let node1 = storage.create_node("person", HashMap::new()).unwrap();
        let node2 = storage.create_node("person", HashMap::new()).unwrap();
        let edge = storage
            .create_edge("knows", &node1.id, &node2.id, HashMap::new())
            .unwrap();

        storage.drop_edge(&edge.id).unwrap();

        assert!(storage.get_edge(&edge.id).is_err());
    }

    #[test]
    fn test_check_exists() {
        let (storage, _temp_dir) = setup_temp_db();

        let node = storage.create_node("person", HashMap::new()).unwrap();
        assert!(storage.check_exists(&node.id).unwrap());
        assert!(!storage.check_exists("nonexistent").unwrap());
    }

    #[test]
    fn test_get_temp_node() {
        let (storage, _temp_dir) = setup_temp_db();

        let node = storage.create_node("person", HashMap::new()).unwrap();

        let temp_node = storage.get_temp_node(&node.id).unwrap();

        assert_eq!(node.id, temp_node.id);
        assert_eq!(node.label, temp_node.label);
    }

    #[test]
    fn test_multiple_edges_between_nodes() {
        let (storage, _temp_dir) = setup_temp_db();

        let node1 = storage.create_node("person", HashMap::new()).unwrap();
        let node2 = storage.create_node("person", HashMap::new()).unwrap();

        let edge1 = storage
            .create_edge("knows", &node1.id, &node2.id, HashMap::new())
            .unwrap();
        let edge2 = storage
            .create_edge("likes", &node1.id, &node2.id, HashMap::new())
            .unwrap();

        assert!(storage.get_edge(&edge1.id).is_ok());
        assert!(storage.get_edge(&edge2.id).is_ok());
    }

    #[test]
    fn test_node_with_properties() {
        let (storage, _temp_dir) = setup_temp_db();

        let mut properties = HashMap::new();
        properties.insert("name".to_string(), Value::String("George".to_string()));
        properties.insert("age".to_string(), Value::Integer(22));
        properties.insert("active".to_string(), Value::Boolean(true));

        let node = storage.create_node("person", properties).unwrap();
        let retrieved_node = storage.get_node(&node.id).unwrap();

        assert_eq!(
            retrieved_node.properties.get("name").unwrap(),
            &Value::String("George".to_string())
        );
        assert_eq!(
            retrieved_node.properties.get("age").unwrap(),
            &Value::Integer(22)
        );
        assert_eq!(
            retrieved_node.properties.get("active").unwrap(),
            &Value::Boolean(true)
        );
    }

    fn test_get_all_nodes() {
        let (storage, _temp_dir) = setup_temp_db();
        let node1 = storage.create_node("person", HashMap::new()).unwrap();
        let node2 = storage.create_node("thing", HashMap::new()).unwrap();
        let node3 = storage.create_node("other", HashMap::new()).unwrap();

        let nodes = storage.get_all_nodes().unwrap();

        assert_eq!(nodes.len(), 3);

        let node_ids: Vec<String> = nodes.iter().map(|n| n.id.clone()).collect();

        assert!(node_ids.contains(&node1.id));
        assert!(node_ids.contains(&node2.id));
        assert!(node_ids.contains(&node3.id));

        let labels: Vec<String> = nodes.iter().map(|n| n.label.clone()).collect();

        assert!(labels.contains(&"person".to_string()));
        assert!(labels.contains(&"thing".to_string()));
        assert!(labels.contains(&"other".to_string()));
    }

    #[test]
fn test_get_all_edges() {
    let (storage, _temp_dir) = setup_temp_db();
    
    let node1 = storage.create_node("person", HashMap::new()).unwrap();
    let node2 = storage.create_node("person", HashMap::new()).unwrap();
    let node3 = storage.create_node("person", HashMap::new()).unwrap();

    let edge1 = storage.create_edge("knows", &node1.id, &node2.id, HashMap::new()).unwrap();
    let edge2 = storage.create_edge("likes", &node2.id, &node3.id, HashMap::new()).unwrap();
    let edge3 = storage.create_edge("follows", &node1.id, &node3.id, HashMap::new()).unwrap();

    let edges = storage.get_all_edges().unwrap();

    assert_eq!(edges.len(), 3);

    let edge_ids: Vec<String> = edges.iter()
        .map(|e| e.id.clone())
        .collect();

    assert!(edge_ids.contains(&edge1.id));
    assert!(edge_ids.contains(&edge2.id));
    assert!(edge_ids.contains(&edge3.id));

    let labels: Vec<String> = edges.iter()
        .map(|e| e.label.clone())
        .collect();

    assert!(labels.contains(&"knows".to_string()));
    assert!(labels.contains(&"likes".to_string()));
    assert!(labels.contains(&"follows".to_string()));

    let connections: Vec<(String, String)> = edges.iter()
        .map(|e| (e.from_node.clone(), e.to_node.clone()))
        .collect();

    assert!(connections.contains(&(node1.id.clone(), node2.id.clone()))); 
    assert!(connections.contains(&(node2.id.clone(), node3.id.clone()))); 
    assert!(connections.contains(&(node1.id.clone(), node3.id.clone())));
}
}
