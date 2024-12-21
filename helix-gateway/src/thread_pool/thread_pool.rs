use flume::{bounded, unbounded, Receiver, Sender};
use helix_engine::graph_core::graph_core::HelixGraphEngine;
use helix_engine::storage_core::storage_core::HelixGraphStorage; // change once transactions in place
use helix_engine::types::GraphError;
use std::io::Read;
use std::net::TcpStream;
use std::sync::{Arc, Mutex};
use std::thread;

use protocol::request::Request;
use protocol::response::Response;
use crate::router::router::HelixRouter;

pub struct Worker {
    id: usize,
    thread: thread::JoinHandle<()>,
    // pub reciever: Arc<Mutex<Receiver<TcpStream>>>,
}

impl Worker {
    fn new(
        id: usize,
        graph_access: Arc<Mutex<HelixGraphEngine>>,
        router: Arc<HelixRouter>,
        rx: Arc<Mutex<Receiver<TcpStream>>>,
    ) -> Arc<Worker> {
        Arc::new(Worker {
            id,
            thread: thread::spawn(move || loop {
                let mut conn = rx.lock().unwrap().recv().unwrap(); // TODO: Handle error
                let request = Request::from_stream(&mut conn).unwrap(); // TODO: Handle Error
                let mut response = Response::new();
                router.handle(Arc::clone(&graph_access), request, &mut response).unwrap(); // TODO: Handle Error
                response.send(&mut conn).unwrap();
            }),
        })
    }
}

pub struct ThreadPool {
    pub sender: Sender<TcpStream>,
    pub num_unused_workers: Mutex<usize>,
    pub num_used_workers: Mutex<usize>,
    pub workers: Mutex<Vec<Arc<Worker>>>,
}

impl ThreadPool {
    pub fn new(
        size: usize,
        storage: HelixGraphEngine,
        router: Arc<HelixRouter>,
    ) -> Self {
        assert!(
            size > 0,
            "Expected number of threads in thread pool to be more than 0, got {}",
            size
        );
        let mut workers = Vec::with_capacity(size);
        let (tx, rx) = flume::unbounded::<TcpStream>();
        let graph = Arc::new(Mutex::new(storage));
        let reciever = Arc::new(Mutex::new(rx));
        for id in 0..size {
            workers.push(Worker::new(
                id,
                Arc::clone(&graph),
                Arc::clone(&router),
                Arc::clone(&reciever),
            ));
        }
        ThreadPool {
            sender: tx,
            num_unused_workers: Mutex::new(workers.len()),
            num_used_workers: Mutex::new(0),
            // used_workers: Mutex::new(Vec::with_capacity(workers.len())),
            workers: Mutex::new(workers),
        }
    }
}
