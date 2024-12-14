use crate::storage_core::{storage_core::HelixGraphStorage, storage_methods::StorageMethods};

pub trait SourceTraversalSteps {
    fn v(&mut self, storage: &HelixGraphStorage) -> &mut Self;
    fn e(&mut self, storage: &HelixGraphStorage) -> &mut Self;

    fn add_v(&mut self, storage: &HelixGraphStorage, node_label: &str) -> &mut Self;
    fn add_e(&mut self, storage: &HelixGraphStorage, edge_label: &str, from_id: &str, to_id: &str) -> &mut Self;
}

pub trait TraversalSteps {

    fn out(&mut self, storage: &HelixGraphStorage, edge_label: &str) -> &mut Self;
    fn out_e(&mut self, storage: &HelixGraphStorage, edge_label: &str) -> &mut Self;
    
    fn in_(&mut self, storage: &HelixGraphStorage, edge_label: &str) -> &mut Self;
    fn in_e(&mut self, storage: &HelixGraphStorage, edge_label: &str) -> &mut Self;

}