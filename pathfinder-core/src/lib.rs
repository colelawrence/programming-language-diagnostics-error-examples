use petgraph::algo::dijkstra;
use petgraph::graph::{Graph, NodeIndex};
use shared_types::{Edge, PathResult, Point};
use std::collections::HashMap;

pub mod handler;
pub use handler::PathfinderHandler;

/// Pure function to compute Euclidean distance between two points
pub fn euclidean_distance(p1: &Point, p2: &Point) -> f64 {
    ((p2.x - p1.x).powi(2) + (p2.y - p1.y).powi(2)).sqrt()
}

/// Pure function to reconstruct the shortest path from Dijkstra results
pub fn reconstruct_path(
    graph: &Graph<(), f64, petgraph::Undirected>,
    distances: &HashMap<NodeIndex, f64>,
    start: NodeIndex,
    end: NodeIndex,
) -> Vec<NodeIndex> {
    let mut path = vec![end];
    let mut current = end;

    while current != start {
        let current_dist = distances[&current];

        let prev = graph
            .neighbors(current)
            .find(|&neighbor| {
                if let Some(&neighbor_dist) = distances.get(&neighbor) {
                    if let Some(edge) = graph.find_edge(neighbor, current) {
                        let edge_weight = graph[edge];
                        (neighbor_dist + edge_weight - current_dist).abs() < 1e-10
                    } else {
                        false
                    }
                } else {
                    false
                }
            })
            .expect("Path reconstruction failed");

        path.push(prev);
        current = prev;
    }

    path.reverse();
    path
}

/// Core business logic: compute the shortest path
/// This is completely transport-agnostic
pub fn compute_shortest_path(
    points: &[Point],
    edges: &[Edge],
    start_idx: usize,
    end_idx: usize,
) -> Result<PathResult, String> {
    // Build the graph
    let mut graph: Graph<(), f64, petgraph::Undirected> = Graph::new_undirected();

    let nodes: Vec<NodeIndex> = (0..points.len()).map(|_| graph.add_node(())).collect();

    for edge in edges {
        let distance = euclidean_distance(&points[edge.from], &points[edge.to]);
        graph.add_edge(nodes[edge.from], nodes[edge.to], distance);
    }

    let start_node = nodes[start_idx];
    let end_node = nodes[end_idx];

    // Run Dijkstra's algorithm
    let result = dijkstra(&graph, start_node, Some(end_node), |e| *e.weight());

    if let Some(&distance) = result.get(&end_node) {
        let path = reconstruct_path(&graph, &result, start_node, end_node);
        let path_indices: Vec<usize> = path.iter().map(|&n| n.index()).collect();

        Ok(PathResult {
            path: path_indices,
            distance,
        })
    } else {
        Err("No path found".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_euclidean_distance() {
        let p1 = Point { x: 0.0, y: 0.0 };
        let p2 = Point { x: 3.0, y: 4.0 };
        assert_eq!(euclidean_distance(&p1, &p2), 5.0);
    }

    #[test]
    fn test_compute_shortest_path_simple() {
        let points = vec![
            Point { x: 0.0, y: 0.0 },
            Point { x: 1.0, y: 0.0 },
            Point { x: 0.0, y: 1.0 },
        ];

        let edges = vec![
            Edge { from: 0, to: 1 },
            Edge { from: 1, to: 2 },
            Edge { from: 0, to: 2 },
        ];

        let result = compute_shortest_path(&points, &edges, 0, 2);
        assert!(result.is_ok());

        let path_result = result.unwrap();
        assert_eq!(path_result.path, vec![0, 2]);
        assert!((path_result.distance - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_compute_shortest_path_no_path() {
        let points = vec![
            Point { x: 0.0, y: 0.0 },
            Point { x: 1.0, y: 0.0 },
            Point { x: 10.0, y: 10.0 },
        ];

        let edges = vec![Edge { from: 0, to: 1 }];

        let result = compute_shortest_path(&points, &edges, 0, 2);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "No path found");
    }
}
