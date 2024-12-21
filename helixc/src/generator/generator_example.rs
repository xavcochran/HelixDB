use std::collections::HashMap;
use std::error::Error;
use std::path::PathBuf;

use crate::generator::project_gen::ProjectGenerator;
use crate::generator::query_gen::TraversalGenerator;

pub fn run_generator() -> Result<(), Box<dyn Error>> {
    let output_dir = PathBuf::from("../");
    let mut queries = HashMap::new();

    let traversal_generator = TraversalGenerator::new("test_function")
        .v()
        .out("knows")
        .in_("follows")
        .out_e("likes");

    // Generate the traversal code
    let traversal_code = traversal_generator.generate_code()?;
    queries.insert("test_function".to_string(), traversal_code);

    let traversal_generator = TraversalGenerator::new("test_function2")
        .v()
        .out("knows");
    let traversal_code = traversal_generator.generate_code()?;
    queries.insert("test_function2".to_string(), traversal_code);

    // Create and configure the project generator
    let project_generator =
        ProjectGenerator::new("graph_queries", output_dir).with_queries(queries);

    // Generate the project
    project_generator.generate()?;

    println!("Successfully generated project in ./graph_queries");
    println!("To run the generated project:");
    println!("  cd graph_queries");
    println!("  cargo build");

    Ok(())
}

