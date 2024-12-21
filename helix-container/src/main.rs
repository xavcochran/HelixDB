use std::{collections::HashMap, sync::Arc};

use chrono::Utc;
use helix_engine::graph_core::graph_core::HelixGraphEngine;
use helix_gateway::{
    router::router::{HandlerFn, HandlerInput, HandlerSubmission, RouterError},
    GatewayOpts, HelixGateway,
};
use inventory;
use protocol::response::Response;
pub mod traversals;

fn main() {
    let path = format!("../graph_data/{}", Utc::now());
    let graph = HelixGraphEngine::new(path.as_str()).unwrap();
    let routes = HashMap::from_iter(
        inventory::iter::<HandlerSubmission>
            .into_iter()
            .map(|submission| {
                let handler = &submission.0;
                let func: Arc<
                    dyn Fn(&HandlerInput, &mut Response) -> Result<(), RouterError> + Send + Sync,
                > = Arc::new(move |input, response| (handler.func)(input, response));
                (("get".to_ascii_uppercase().to_string(), format!("/{}", handler.name.to_string())), func)
            })
            .collect::<Vec<((String, String), HandlerFn)>>(),
    );
    let gateway = HelixGateway::new(
        "127.0.0.1:1234",
        graph,
        GatewayOpts::DEFAULT_POOL_SIZE,
        Some(routes),
    );

    let _ = gateway.connection_handler.accept_conns().join().unwrap();
}

