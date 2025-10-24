use shared_types_proc::protocol;

pub mod context;
pub mod receiver;
pub mod router;
pub mod storage;

/// A 2D point with x and y coordinates
#[protocol("wasm")]
pub struct Point {
    pub x: f64,
    pub y: f64,
}

/// An edge connecting two points
#[protocol("wasm")]
pub struct Edge {
    pub from: usize,
    pub to: usize,
}

/// Result of a shortest path computation
#[protocol("wasm")]
pub struct PathResult {
    pub path: Vec<usize>,
    pub distance: f64,
}

/// Parameters for finding the shortest path between two points
#[protocol("wasm")]
#[codegen(fn = "find_shortest_path() -> PathResult")]
pub struct ShortestPathParams {
    pub points: Vec<Point>,
    pub edges: Vec<Edge>,
    pub start_idx: usize,
    pub end_idx: usize,
}

/// Graph statistics and metrics
#[protocol("wasm")]
pub struct GraphMetrics {
    pub node_count: usize,
    pub edge_count: usize,
    pub total_edge_length: f64,
    pub avg_edge_length: f64,
}

/// Parameters for computing graph metrics
#[protocol("wasm")]
#[codegen(fn = "compute_graph_metrics() -> GraphMetrics")]
pub struct GraphMetricsParams {
    pub points: Vec<Point>,
    pub edges: Vec<Edge>,
}

#[cfg(test)]
#[cfg(feature = "codegen")]
mod generate {
    use std::{path::PathBuf, process::Command};
    
    #[test]
    #[ignore]
    fn generate_typescript() {
        let cargo_dir = std::env::var("CARGO_MANIFEST_DIR")
            .unwrap()
            .parse::<PathBuf>()
            .unwrap();

        let typescript_generation = derive_codegen::Generation::for_tag("protocol-wasm");

        let mut typescript_command = Command::new("bun");
        typescript_command
            .arg("../generators/generateTypescript.ts")
            .current_dir(&cargo_dir);

        typescript_generation
            .pipe_into(&mut typescript_command)
            .with_output_path(cargo_dir.join("../dist-types"))
            .write();

        // Generate router_gen.rs
        let mut rust_command = Command::new("bun");
        rust_command
            .arg("../generators/generateRustRouterSimple.ts")
            .current_dir(&cargo_dir);

        derive_codegen::Generation::for_tag("protocol-wasm")
            .pipe_into(&mut rust_command)
            .with_output_path(cargo_dir.join("src/router"))
            .write();
    }
}
