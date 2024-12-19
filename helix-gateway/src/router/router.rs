// router

// takes in raw [u8] data
// parses to request type

// then locks graph and passes parsed data and graph to handler to execute query

// returns response

use core::fmt;
use helix_engine::storage_core::storage_core::HelixGraphStorage;
use std::{
    collections::HashMap,
    convert::Infallible,
    ops::Deref,
    sync::{Arc, Mutex},
};

use super::{request::Request, response::Response};

pub struct HandlerInput {
    request: Request,
    graph: Arc<Mutex<HelixGraphStorage>>,
}

type HandlerFn =
    Arc<dyn (Fn(&HandlerInput, &mut Response) -> Result<(), RouterError>) + Send + Sync>;

pub struct HelixRouter {
    /// Method+Path => Function
    pub routes: HashMap<(String, String), HandlerFn>,
}

impl HelixRouter {
    pub fn new() -> Self {
        Self {
            routes: HashMap::new(),
        }
    }

    pub fn add_route<F>(&mut self, method: &str, path: &str, handler: F)
    where
        F: (Fn(&HandlerInput, &mut Response) -> Result<(), RouterError>) + Send + Sync + 'static,
    {
        self.routes
            .insert((method.to_uppercase(), path.to_string()), Arc::new(handler));
    }

    pub fn handle(
        &self,
        graph_access: Arc<Mutex<HelixGraphStorage>>,
        request: Request,
        response: &mut Response
    ) -> Result<(), RouterError> {
        // find route
        let route_key = (request.method.clone(), request.path.clone());
        let handler = match self.routes.get(&route_key) {
            Some(handle) => handle,
            None => {
                response.status=404;   
                println!("{:?}", response);
                return Ok(());
            }
        };

        // get hold of graph storage
        // let graph = &graph_access.lock().unwrap();
        let input = HandlerInput {
            request,
            graph: Arc::clone(&graph_access),
        };

        handler(&input, response)
    }
}

#[derive(Debug)]
pub enum RouterError {
    Io(std::io::Error),
    New(String),
}

impl fmt::Display for RouterError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RouterError::Io(e) => write!(f, "IO error: {}", e),
            RouterError::New(msg) => write!(f, "Graph error: {}", msg),
        }
    }
}
