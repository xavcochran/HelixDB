use std::collections::{HashMap, HashSet};

use bincode::{deserialize, serialize};
use rocksdb::{
    BlockBasedOptions, Cache, ColumnFamilyDescriptor, Direction, IteratorMode, Options, ReadOptions, WriteBatch, WriteBatchWithTransaction, WriteOptions, DB
};

use uuid::Uuid;

use crate::storage_core::storage_methods::StorageMethods;
use crate::types::GraphError;
use protocol::{Edge, Node, Value};
use rayon::*;

const CF_NODES: &str = "nodes"; // For node data (n:)
const CF_EDGES: &str = "edges"; // For edge data (e:)
const CF_INDICES: &str = "indices"; // For all indices (nl:, el:, o:, i:)

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
        // set cache
        let mut opts = Options::default();

        // Basic options
        opts.create_if_missing(true);
        opts.create_missing_column_families(true);
        opts.increase_parallelism(num_cpus::get() as i32);
        opts.set_max_background_jobs(8);

        // Write path optimizations
        opts.set_write_buffer_size(256 * 1024 * 1024); // 256MB write buffer
        opts.set_max_write_buffer_number(4);
        opts.set_min_write_buffer_number_to_merge(2);
        opts.set_level_zero_file_num_compaction_trigger(4);
        opts.set_level_zero_slowdown_writes_trigger(20);
        opts.set_level_zero_stop_writes_trigger(36);

        // Configure compaction
        opts.set_disable_auto_compactions(false);
        opts.set_target_file_size_base(64 * 1024 * 1024); // 64MB
        opts.set_target_file_size_multiplier(1);
        opts.set_max_bytes_for_level_base(512 * 1024 * 1024); // 512MB
        opts.set_max_bytes_for_level_multiplier(8.0);

        // Setup column families with specific options
        let mut node_opts = Options::default();
        let mut edge_opts = Options::default();
        let mut index_opts = Options::default();

        // Node CF optimizations
        let node_cache = Cache::new_lru_cache(1 * 1024 * 1024 * 1024); // 4GB cache
        let mut node_block_opts = BlockBasedOptions::default();
        node_block_opts.set_block_cache(&node_cache);
        node_block_opts.set_block_size(32 * 1024); // 32KB blocks
        node_block_opts.set_cache_index_and_filter_blocks(true);
        node_block_opts.set_bloom_filter(10.0, false);
        node_opts.set_block_based_table_factory(&node_block_opts);

        // Edge CF optimizations
        let edge_cache = Cache::new_lru_cache(2 * 1024 * 1024 * 1024); // 8GB cache
        let mut edge_block_opts = BlockBasedOptions::default();
        edge_block_opts.set_block_cache(&edge_cache);
        edge_block_opts.set_block_size(64 * 1024); // 64KB blocks
        edge_block_opts.set_cache_index_and_filter_blocks(true);
        edge_block_opts.set_bloom_filter(10.0, false);
        edge_opts.set_block_based_table_factory(&edge_block_opts);

        // Index CF optimizations (for edge indices)
        let index_cache = Cache::new_lru_cache(1 * 1024 * 1024 * 1024); // 2GB cache
        let mut index_block_opts = BlockBasedOptions::default();
        index_block_opts.set_block_cache(&index_cache);
        index_block_opts.set_block_size(16 * 1024); // 16KB blocks
        index_block_opts.set_cache_index_and_filter_blocks(true);
        index_block_opts.set_bloom_filter(10.0, false);
        index_opts.set_block_based_table_factory(&index_block_opts);

        let cf_descriptors = vec![
            ColumnFamilyDescriptor::new(CF_NODES, node_opts),
            ColumnFamilyDescriptor::new(CF_EDGES, edge_opts),
            ColumnFamilyDescriptor::new(CF_INDICES, index_opts),
        ];

        let db = match DB::open_cf_descriptors(&opts, path, cf_descriptors) {
            Ok(db) => db,
            Err(err) => return Err(GraphError::from(err)),
        };
        Ok(Self { db })
    }

    /// Creates node key using the prefix and given id
    pub fn node_key(id: &str) -> Vec<u8> {
        [NODE_PREFIX, id.as_bytes()].concat()
    }

    /// Creates edge key using the prefix and given id
    pub fn edge_key(id: &str) -> Vec<u8> {
        [EDGE_PREFIX, id.as_bytes()].concat()
    }

    /// Creates node label key using the prefix, the given label, and id
    pub fn node_label_key(label: &str, id: &str) -> Vec<u8> {
        [NODE_LABEL_PREFIX, label.as_bytes(), b":", id.as_bytes()].concat()
    }

    /// Creates edge label key using the prefix, the given label, and  id
    pub fn edge_label_key(label: &str, id: &str) -> Vec<u8> {
        [EDGE_LABEL_PREFIX, label.as_bytes(), b":", id.as_bytes()].concat()
    }

    /// Creates key for an outgoing edge using the prefix, source node id, and edge id
    /// 35 Bytes
    pub fn out_edge_key(source_node_id: &str, edge_id: &str) -> Vec<u8> {
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
    pub fn in_edge_key(sink_node_id: &str, edge_id: &str) -> Vec<u8> {
        [
            IN_EDGES_PREFIX,
            sink_node_id.as_bytes(),
            b":",
            edge_id.as_bytes(),
        ]
        .concat()
    }
}

impl StorageMethods for HelixGraphStorage {
    fn check_exists(&self, id: &str) -> Result<bool, GraphError> {
        let cf_nodes = self.db.cf_handle(CF_NODES).unwrap();
        match self
            .db
            .get_pinned_cf(cf_nodes, [NODE_PREFIX, id.as_bytes()].concat())
        {
            Ok(Some(_)) => Ok(true),
            Ok(None) => Ok(false),
            Err(err) => Err(GraphError::from(err)),
        }
    }

    fn get_temp_node(&self, id: &str) -> Result<Node, GraphError> {
        let cf_nodes = self.db.cf_handle(CF_NODES).unwrap();
        match self
            .db
            .get_pinned_cf(cf_nodes, [NODE_PREFIX, id.as_bytes()].concat())
        {
            Ok(Some(data)) => Ok(deserialize(&data).unwrap()),
            Ok(None) => Err(GraphError::New(format!("Item not found!"))),
            Err(err) => Err(GraphError::from(err)),
        }
    }

    fn get_temp_edge(&self, id: &str) -> Result<Edge, GraphError> {
        let cf_edges = self.db.cf_handle(CF_EDGES).unwrap();
        match self
            .db
            .get_pinned_cf(cf_edges, [EDGE_PREFIX, id.as_bytes()].concat())
        {
            Ok(Some(data)) => Ok(deserialize(&data).unwrap()),
            Ok(None) => Err(GraphError::New(format!("Item not found!"))),
            Err(err) => Err(GraphError::from(err)),
        }
    }

    fn get_node(&self, id: &str) -> Result<Node, GraphError> {
        let cf_nodes = self.db.cf_handle(CF_NODES).unwrap();
        match self
            .db
            .get_cf(cf_nodes, [NODE_PREFIX, id.as_bytes()].concat())
        {
            Ok(Some(data)) => Ok(deserialize(&data).unwrap()),
            Ok(None) => Err(GraphError::New(format!("Item not found!"))),
            Err(err) => Err(GraphError::from(err)),
        }
    }
    fn get_edge(&self, id: &str) -> Result<Edge, GraphError> {
        let cf_edges = self.db.cf_handle(CF_EDGES).unwrap();
        match self
            .db
            .get_cf(cf_edges, [EDGE_PREFIX, id.as_bytes()].concat())
        {
            Ok(Some(data)) => Ok(deserialize(&data).unwrap()),
            Ok(None) => Err(GraphError::New(format!("Item not found!"))),
            Err(err) => Err(GraphError::from(err)),
        }
    }

    fn get_out_edges(&self, node_id: &str, edge_label: &str) -> Result<Vec<Edge>, GraphError> {
        let cf_edge_index = self.db.cf_handle(CF_EDGES).unwrap();

        let mut edges = Vec::new();

        let mut read_opts = ReadOptions::default();
        read_opts.set_verify_checksums(false);
        read_opts.set_readahead_size(2 * 1024 * 1024);
        read_opts.set_prefix_same_as_start(true);

        let out_prefix = Self::out_edge_key(node_id, "");
        let iter = self.db.iterator_cf_opt(
            cf_edge_index,
            read_opts,
            IteratorMode::From(&out_prefix, rocksdb::Direction::Forward),
        );

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
        let cf_edges = self.db.cf_handle(CF_EDGES).unwrap();
        let mut edges = Vec::with_capacity(20);
        // get in edges
        let in_prefix = Self::in_edge_key(node_id, "");
        let mut read_opts = ReadOptions::default();
        read_opts.set_verify_checksums(false);
        read_opts.set_readahead_size(2 * 1024 * 1024);
        read_opts.set_prefix_same_as_start(true);
        let iter = self.db.iterator_cf_opt(
            cf_edges,
            read_opts,
            IteratorMode::From(&in_prefix, rocksdb::Direction::Forward),
        );

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

    fn get_out_nodes(&self, node_id: &str, edge_label: &str) -> Result<Vec<Node>, GraphError> {
        let cf_edges = self.db.cf_handle(CF_EDGES).unwrap();
        let mut nodes = Vec::with_capacity(20);

        // // Prefetch out edges
        let out_prefix = Self::out_edge_key(node_id, "");
        let mut read_opts = ReadOptions::default();
        read_opts.set_verify_checksums(false);
        read_opts.set_readahead_size(2 * 1024 * 1024);
        read_opts.set_prefix_same_as_start(true);
        let iter = self.db.iterator_cf_opt(
            cf_edges,
            read_opts,
            IteratorMode::From(&out_prefix, rocksdb::Direction::Forward),
        );

        for result in iter {
            let (key, _) = result?;
            if !key.starts_with(&out_prefix) {
                break;
            }
            let edge =
                &self.get_temp_edge(&std::str::from_utf8(&key[out_prefix.len()..]).unwrap())?;

            if edge.label == edge_label {
                if let Ok(node) = self.get_node(&edge.to_node) {
                    nodes.push(node);
                }
            }
        }

        Ok(nodes)
    }

    fn get_in_nodes(&self, node_id: &str, edge_label: &str) -> Result<Vec<Node>, GraphError> {
        let cf_edges = self.db.cf_handle(CF_EDGES).unwrap();
        let mut nodes = Vec::with_capacity(20);

        // Prefetch in edges
        let in_prefix = Self::in_edge_key(node_id, "");
        let mut read_opts = ReadOptions::default();
        read_opts.set_verify_checksums(false);
        read_opts.set_readahead_size(2 * 1024 * 1024);
        read_opts.set_prefix_same_as_start(true);
        let iter = self.db.iterator_cf_opt(
            cf_edges,
            read_opts,
            IteratorMode::From(&in_prefix, rocksdb::Direction::Forward),
        );

        for result in iter {
            let (key, _) = result?;
            if !key.starts_with(&in_prefix) {
                break;
            }
            let edge =
                &self.get_temp_edge(&std::str::from_utf8(&key[in_prefix.len()..]).unwrap())?;

            if edge.label == edge_label {
                if let Ok(node) = self.get_node(&edge.from_node) {
                    nodes.push(node);
                }
            }
        }

        Ok(nodes)
    }

    fn get_all_nodes(&self) -> Result<Vec<Node>, GraphError> {
        let node_prefix = Self::node_key("");
        let mut nodes = Vec::new();

        let cf_nodes = self.db.cf_handle(CF_NODES).unwrap();

        let mut read_opts = ReadOptions::default();
        read_opts.set_verify_checksums(false);
        read_opts.set_readahead_size(2 * 1024 * 1024);
        read_opts.set_prefix_same_as_start(true);
        let iter = self.db.iterator_cf_opt(
            cf_nodes,
            read_opts,
            IteratorMode::From(&node_prefix, rocksdb::Direction::Forward),
        );

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

        let cf_edges = self.db.cf_handle(CF_EDGES).unwrap();
        let mut read_opts = ReadOptions::default();
        read_opts.set_verify_checksums(false);
        read_opts.set_readahead_size(2 * 1024 * 1024);
        read_opts.set_prefix_same_as_start(true);
        let iter = self.db.iterator_cf_opt(
            cf_edges,
            read_opts,
            IteratorMode::From(&edge_prefix, rocksdb::Direction::Forward),
        );

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
        properties: impl IntoIterator<Item = (String, Value)>,
    ) -> Result<Node, GraphError> {
        let node = Node {
            id: Uuid::new_v4().to_string(),
            label: label.to_string(),
            properties: HashMap::from_iter(properties),
        };
        let cf_nodes = self.db.cf_handle(CF_NODES).unwrap();
        let mut new_batch = WriteBatchWithTransaction::default();

        new_batch.put_cf(
            cf_nodes,
            Self::node_key(&node.id),
            serialize(&node).unwrap(),
        );
        new_batch.put_cf(cf_nodes, Self::node_label_key(label, &node.id), vec![]);

        self.db.write(new_batch)?;
        Ok(node)
    }

    fn create_edge(
        &self,
        label: &str,
        from_node: &str,
        to_node: &str,
        properties: impl IntoIterator<Item = (String, Value)>,
    ) -> Result<Edge, GraphError> {
        // look at creating check function that uses pinning
        // let (from_exists, to_exists) = rayon::join(
        //     || self.get_temp_node(from_node).is_ok(),
        //     || self.get_temp_node(to_node).is_ok(),
        // );

        // if !from_exists || !to_exists {
        //     return Err(GraphError::New(
        //         "One or both nodes do not exist".to_string(),
        //     ));
        // }

        if !self.get_node(from_node).is_ok() || !self.get_node(to_node).is_ok() {
            return Err(GraphError::New(format!("One or both nodes do not exist")));
        }

        let edge = Edge {
            id: Uuid::new_v4().to_string(),
            label: label.to_string(),
            from_node: from_node.to_string(),
            to_node: to_node.to_string(),
            properties: HashMap::from_iter(properties),
        };
        let cf_edges = self.db.cf_handle(CF_EDGES).unwrap();
        let mut batch = WriteBatch::default();

        // new edge
        batch.put_cf(
            cf_edges,
            Self::edge_key(&edge.id),
            bincode::serialize(&edge).unwrap(),
        );
        // edge label
        batch.put_cf(cf_edges, Self::edge_label_key(label, &edge.id), vec![]);

        // edge keys
        batch.put_cf(cf_edges, Self::out_edge_key(from_node, &edge.id), vec![]);
        batch.put_cf(cf_edges, Self::in_edge_key(to_node, &edge.id), vec![]);

        let mut write_opts = WriteOptions::default();
        write_opts.set_sync(false); 
        write_opts.disable_wal(true);

        self.db.write_opt(batch, &write_opts)?;
        // self.db.write(batch)?;
        Ok(edge)
    }

    fn drop_node(&self, id: &str) -> Result<(), GraphError> {
        let cf_nodes = self.db.cf_handle(CF_NODES).unwrap();

        let mut read_opts = ReadOptions::default();
        read_opts.set_verify_checksums(false);
        read_opts.set_readahead_size(2 * 1024 * 1024);
        read_opts.set_prefix_same_as_start(true);
        // get out edges
        let out_prefix = Self::out_edge_key(id, "");
        let iter = self.db.iterator_cf_opt(
            cf_nodes,
            read_opts,
            IteratorMode::From(&out_prefix, rocksdb::Direction::Forward),
        );
        // delete them
        for result in iter {
            let (key, _) = result?;
            if !key.starts_with(&out_prefix) {
                break;
            }

            let edge_id = String::from_utf8(key[out_prefix.len()..].to_vec()).unwrap();
            self.drop_edge(&edge_id)?;
        }

        let cf_edges = self.db.cf_handle(CF_EDGES).unwrap();
        let mut read_opts = ReadOptions::default();
        read_opts.set_verify_checksums(false);
        read_opts.set_readahead_size(2 * 1024 * 1024);
        read_opts.set_prefix_same_as_start(true);
        // get in edges
        let in_prefix = Self::in_edge_key(id, "");
        let iter = self.db.iterator_cf_opt(
            cf_edges,
            read_opts,
            IteratorMode::From(&in_prefix, rocksdb::Direction::Forward),
        );
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
        self.db.delete_cf(cf_nodes, Self::node_key(id))?;

        Ok(())
    }

    fn drop_edge(&self, edge_id: &str) -> Result<(), GraphError> {
        let cf_edges = self.db.cf_handle(CF_EDGES).unwrap();
        let edge_data = self
            .db
            .get_pinned_cf(cf_edges, Self::edge_key(edge_id))?
            .unwrap();
        let edge: Edge = deserialize(&edge_data).unwrap();

        let mut batch = WriteBatch::default();

        batch.delete_cf(cf_edges, Self::out_edge_key(&edge.from_node, edge_id));
        batch.delete_cf(cf_edges, Self::in_edge_key(&edge.to_node, edge_id));
        batch.delete_cf(cf_edges, Self::edge_key(edge_id));

        match self.db.write(batch) {
            Ok(_) => Ok(()),
            Err(err) => Err(GraphError::from(err)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::props;
    use crate::storage_core::storage_methods::StorageMethods;
    use protocol::Value;
    use rocksdb::properties;
    use std::collections::HashMap;
    use std::iter;
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

        let mut properties = props! {
            "name" => "test node",
        };

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

        let node1 = storage.create_node("person", props!()).unwrap();
        let node2 = storage.create_node("person", props!()).unwrap();

        let mut edge_props = props! {
            "age" => 22,
        };

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

        let result = storage.create_edge("knows", "nonexistent1", "nonexistent2", props!());

        assert!(result.is_err());
    }

    #[test]
    fn test_drop_node() {
        let (storage, _temp_dir) = setup_temp_db();

        let node1 = storage.create_node("person", props!()).unwrap();
        let node2 = storage.create_node("person", props!()).unwrap();
        let node3 = storage.create_node("person", props!()).unwrap();

        storage
            .create_edge("knows", &node1.id, &node2.id, props!())
            .unwrap();
        storage
            .create_edge("knows", &node3.id, &node1.id, props!())
            .unwrap();

        storage.drop_node(&node1.id).unwrap();

        assert!(storage.get_node(&node1.id).is_err());
    }

    #[test]
    fn test_drop_edge() {
        let (storage, _temp_dir) = setup_temp_db();

        let node1 = storage.create_node("person", props!()).unwrap();
        let node2 = storage.create_node("person", props!()).unwrap();
        let edge = storage
            .create_edge("knows", &node1.id, &node2.id, props!())
            .unwrap();

        storage.drop_edge(&edge.id).unwrap();

        assert!(storage.get_edge(&edge.id).is_err());
    }

    #[test]
    fn test_check_exists() {
        let (storage, _temp_dir) = setup_temp_db();

        let node = storage.create_node("person", props!()).unwrap();
        assert!(storage.check_exists(&node.id).unwrap());
        assert!(!storage.check_exists("nonexistent").unwrap());
    }

    #[test]
    fn test_get_temp_node() {
        let (storage, _temp_dir) = setup_temp_db();

        let node = storage.create_node("person", props!()).unwrap();

        let temp_node = storage.get_temp_node(&node.id).unwrap();

        assert_eq!(node.id, temp_node.id);
        assert_eq!(node.label, temp_node.label);
    }

    #[test]
    fn test_multiple_edges_between_nodes() {
        let (storage, _temp_dir) = setup_temp_db();

        let node1 = storage.create_node("person", props!()).unwrap();
        let node2 = storage.create_node("person", props!()).unwrap();

        let edge1 = storage
            .create_edge("knows", &node1.id, &node2.id, props!())
            .unwrap();
        let edge2 = storage
            .create_edge("likes", &node1.id, &node2.id, props!())
            .unwrap();

        assert!(storage.get_edge(&edge1.id).is_ok());
        assert!(storage.get_edge(&edge2.id).is_ok());
    }

    #[test]
    fn test_node_with_properties() {
        let (storage, _temp_dir) = setup_temp_db();

        let mut properties = props! {
            "name" => "George",
            "age" => 22,
            "active" => true,
        };
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
        let node1 = storage.create_node("person", props!()).unwrap();
        let node2 = storage.create_node("thing", props!()).unwrap();
        let node3 = storage.create_node("other", props!()).unwrap();

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

        let node1 = storage.create_node("person", props!()).unwrap();
        let node2 = storage.create_node("person", props!()).unwrap();
        let node3 = storage.create_node("person", props!()).unwrap();

        let edge1 = storage
            .create_edge("knows", &node1.id, &node2.id, props!())
            .unwrap();
        let edge2 = storage
            .create_edge("likes", &node2.id, &node3.id, props!())
            .unwrap();
        let edge3 = storage
            .create_edge("follows", &node1.id, &node3.id, props!())
            .unwrap();

        let edges = storage.get_all_edges().unwrap();

        assert_eq!(edges.len(), 3);

        let edge_ids: Vec<String> = edges.iter().map(|e| e.id.clone()).collect();

        assert!(edge_ids.contains(&edge1.id));
        assert!(edge_ids.contains(&edge2.id));
        assert!(edge_ids.contains(&edge3.id));

        let labels: Vec<String> = edges.iter().map(|e| e.label.clone()).collect();

        assert!(labels.contains(&"knows".to_string()));
        assert!(labels.contains(&"likes".to_string()));
        assert!(labels.contains(&"follows".to_string()));

        let connections: Vec<(String, String)> = edges
            .iter()
            .map(|e| (e.from_node.clone(), e.to_node.clone()))
            .collect();

        assert!(connections.contains(&(node1.id.clone(), node2.id.clone())));
        assert!(connections.contains(&(node2.id.clone(), node3.id.clone())));
        assert!(connections.contains(&(node1.id.clone(), node3.id.clone())));
    }
}
