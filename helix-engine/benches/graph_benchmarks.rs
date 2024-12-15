use criterion::{black_box, criterion_group, criterion_main, Criterion};
use rand::Rng;
use std::time::Duration;
use tempfile::TempDir;
use helix_engine::{
    graph_core::{traversal::TraversalBuilder, traversal_steps::{SourceTraversalSteps, TraversalSteps}}, props, storage_core::{storage_core::HelixGraphStorage, storage_methods::StorageMethods}
};

fn create_test_graph(size: usize, edges_per_node: usize) -> (HelixGraphStorage, TempDir, Vec<String>) {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().to_str().unwrap();
    let storage = HelixGraphStorage::new(db_path).unwrap();
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

    (storage, temp_dir, node_ids)
}

fn bench_graph_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("graph_operations");
    group.measurement_time(Duration::from_secs(10));
    group.sample_size(10);

    for size in [100, 1000, 10000].iter() {
        let edges_per_node = 5;
        
        // Benchmark node creation
        group.bench_function(format!("create_nodes_{}", size), |b| {
            b.iter(|| {
                let (storage, _temp_dir, _) = create_test_graph(*size, 0);
                black_box(storage)
            });
        });

        // Benchmark edge creation
        group.bench_function(format!("create_edges_{}", size), |b| {
            b.iter(|| {
                let (storage, _temp_dir, _) = create_test_graph(*size, edges_per_node);
                black_box(storage)
            });
        });

        let (storage, _temp_dir, node_ids) = create_test_graph(*size, edges_per_node);
        let start_node = storage.get_node(&node_ids[0]).unwrap();

        // Benchmark simple traversals
        group.bench_function(format!("out_traversal_{}", size), |b| {
            b.iter(|| {
                let mut traversal = TraversalBuilder::new(vec![start_node.clone()]);
                traversal.out(&storage, "knows");
                black_box(traversal)
            });
        });

        // Benchmark chained traversals
        group.bench_function(format!("chained_traversal_{}", size), |b| {
            b.iter(|| {
                let mut traversal = TraversalBuilder::new(vec![start_node.clone()]);
                traversal
                    .out(&storage, "knows")
                    .out(&storage, "knows")
                    .out(&storage, "knows");
                black_box(traversal)
            });
        });

        // Benchmark full graph scan
        group.bench_function(format!("full_graph_scan_{}", size), |b| {
            b.iter(|| {
                let mut traversal = TraversalBuilder::new(vec![]);
                traversal.v(&storage);
                black_box(traversal)
            });
        });
    }

    group.finish();
}

fn bench_complex_queries(c: &mut Criterion) {
    let mut group = c.benchmark_group("complex_queries");
    group.measurement_time(Duration::from_secs(10));
    group.sample_size(10);

    let size = 100000;
    let edges_per_node = 10;
    let (storage, _temp_dir, node_ids) = create_test_graph(size, edges_per_node);

    let start_node = storage.get_node(&node_ids[0]).unwrap();

    // Benchmark pattern matching query
    group.bench_function("pattern_matching_100000", |b| {
        b.iter(|| {
            let mut traversal = TraversalBuilder::new(vec![start_node.clone()]);
            traversal
                .out(&storage, "knows")
                .out(&storage, "knows");
            black_box(traversal)
        });
    });

    // Benchmark circular traversal
    group.bench_function("circular_traversal_100000", |b| {
        b.iter(|| {
            let mut traversal = TraversalBuilder::new(vec![start_node.clone()]);
            traversal
                .out(&storage, "knows")
                .out(&storage, "knows")
                .out(&storage, "knows")
                .in_(&storage, "knows");
            black_box(traversal)
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_graph_operations,
    bench_complex_queries
);
criterion_main!(benches);