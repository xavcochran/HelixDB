// router

// takes in raw [u8] data
// parses to request type

// then locks graph and passes parsed data and graph to handler to execute query

// returns response

use core::fmt;
use helix_engine::{graph_core::graph_core::HelixGraphEngine, storage_core::storage_core::HelixGraphStorage};
use std::{
    collections::HashMap,
    convert::Infallible,
    ops::Deref,
    sync::{Arc, Mutex},
};

use protocol::{request::Request, response::Response};

pub struct HandlerInput {
    pub request: Request,
    pub graph: Arc<Mutex<HelixGraphEngine>>,
}

// basic type for function pointer
pub type BasicHandlerFn = fn(&HandlerInput, &mut Response) -> Result<(), RouterError>;

// thread safe type for multi threaded use
pub type HandlerFn =
    Arc<dyn Fn(&HandlerInput, &mut Response) -> Result<(), RouterError> + Send + Sync>;

#[derive(Clone, Debug)]
pub struct HandlerSubmission(pub Handler);

#[derive(Clone, Debug)]
pub struct Handler {
    pub name: &'static str,
    pub func: BasicHandlerFn,
}

impl Handler {
    pub const fn new(name: &'static str, func: BasicHandlerFn) -> Self {
        Self { name, func }
    }
}

inventory::collect!(HandlerSubmission);

pub struct HelixRouter {
    /// Method+Path => Function
    pub routes: HashMap<(String, String), HandlerFn>,
}

impl HelixRouter {
    pub fn new(routes: Option<HashMap<(String, String), HandlerFn>>) -> Self {
        let rts = match routes {
            Some(routes) => routes,
            None => HashMap::new(),
        };
        Self { routes: rts }
    }

    pub fn add_route(&mut self, method: &str, path: &str, handler: BasicHandlerFn) {
        self.routes
            .insert((method.to_uppercase(), path.to_string()), Arc::new(handler));
    }

    pub fn handle(
        &self,
        graph_access: Arc<Mutex<HelixGraphEngine>>,
        request: Request,
        response: &mut Response,
    ) -> Result<(), RouterError> {
        let route_key = (request.method.clone(), request.path.clone());
        let handler = match self.routes.get(&route_key) {
            Some(handle) => handle,
            None => {
                response.status = 404;
                return Ok(());
            }
        };

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
