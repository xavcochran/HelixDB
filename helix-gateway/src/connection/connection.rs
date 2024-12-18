use chrono::{DateTime, Utc};
use helix_engine::{storage_core::storage_core::HelixGraphStorage, types::GraphError};
use std::{
    collections::HashMap,
    net::{TcpListener, TcpStream},
    sync::{Arc, Mutex}, time::Instant,
};
use uuid::Uuid;


use crate::{router::router::HelixRouter, thread_pool::thread_pool::ThreadPool};

pub struct ConnectionHandler {
    pub listener: TcpListener,
    active_connections: Arc<Mutex<HashMap<String, ClientConnection>>>,
    pub thread_pool: ThreadPool,
}

pub struct ClientConnection {
    id: String,
    stream: TcpStream,
    last_active: DateTime<Utc>,
}

impl ConnectionHandler {
    pub fn new(
        address: &str,
        storage: HelixGraphStorage,
        size: usize,
        router: HelixRouter,
    ) -> Result<Self, GraphError> {
        let listener = TcpListener::bind(address)
            .map_err(|e| GraphError::GraphConnectionError("Failed to bind".to_string(), e))?;

        Ok(Self {
            listener,
            active_connections: Arc::new(Mutex::new(HashMap::new())),
            thread_pool: ThreadPool::new(size, storage, Arc::new(router)),
        })
    }

    pub fn accept_conns(&self) -> Result<(), GraphError> {
        loop {
            let conn = match self.listener.accept() {
                Ok((conn, _)) => conn,
                Err(err) => {
                    return Err(GraphError::GraphConnectionError(
                        "Failed to accept connection".to_string(),
                        err,
                    ));
                }
            };

            let conn_clone = conn.try_clone().unwrap();
            let client = ClientConnection {
                id: Uuid::new_v4().to_string(),
                stream: conn_clone,
                last_active: Utc::now(),
            };
            // insert into hashmap
            self.active_connections.lock().unwrap().insert(client.id.clone(), client);
            
            // pass conn to thread in thread pool via channel
            self.thread_pool.sender.send(conn).unwrap();

        }
    }
}
