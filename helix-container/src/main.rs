use helix_engine::graph_core::graph_core::HelixGraphEngine;
use helix_gateway::{GatewayOpts, HelixGateway};
use chrono::Utc;
fn main() {
    let path = format!("../graph_data/{}", Utc::now());
    let graph = HelixGraphEngine::new(path.as_str()).unwrap();
    let gateway = HelixGateway::new("127.0.0.1:1234", graph.storage, GatewayOpts::DEFAULT_POOL_SIZE);
    

    let _ = gateway.connection_handler.accept_conns().join().unwrap();
}
