use helix_engine::graph_core::traversal::TraversalBuilder;
use helix_engine::graph_core::traversal_steps::{SourceTraversalSteps, TraversalSteps};
use get_routes::handler;
use helix_engine::props;
use helix_engine::storage_core::storage_core::HelixGraphStorage;
use helix_engine::storage_core::storage_methods::StorageMethods;
use helix_gateway::router::router::{HandlerInput, RouterError};
use inventory;
use protocol::response::Response;
use rand::Rng;

#[handler]
pub fn test_function2(input: &HandlerInput, response: &mut Response) -> Result<(), RouterError> {
    let graph = &input.graph.lock().unwrap();
    create_test_graph(&graph.storage, 10000, 10);
    let mut traversal = TraversalBuilder::new(vec![]);
    traversal.v(&graph.storage);
    traversal.out(&graph.storage, "knows");
    response.body = graph.result_to_utf8(&traversal);
    Ok(())
}

fn create_test_graph(storage: &HelixGraphStorage, size: usize, edges_per_node: usize) {
    let mut node_ids = Vec::with_capacity(size);

    for _ in 0..size {
        let node = storage.create_node("person", props!()).unwrap();
        node_ids.push(node.id);
    }

    let mut rng = rand::thread_rng();
    for from_id in &node_ids {
        for _ in 0..edges_per_node {
            let to_index = rng.gen_range(0..size);
            let to_id = &node_ids[to_index];
        
            if from_id != to_id {
                storage
                    .create_edge("knows", from_id, to_id, props!())
                    .unwrap();
            }
        }
    }
}
