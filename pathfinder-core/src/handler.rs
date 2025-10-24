use crate::compute_shortest_path;
use shared_types::context::Context;
use shared_types::router::{CallHandler, ObserverImpl};
use shared_types::storage::Storage;
use shared_types::{GraphMetrics, GraphMetricsParams, PathResult, ShortestPathParams};
use std::sync::Arc;

/// PathfinderHandler implements the CallHandler trait
/// This is the transport-agnostic business logic handler
pub struct PathfinderHandler<S: Storage> {
    storage: Option<Arc<S>>,
}

impl<S: Storage> PathfinderHandler<S> {
    pub fn new(storage: Option<Arc<S>>) -> Self {
        Self { storage }
    }
}

impl<S: Storage> CallHandler for PathfinderHandler<S> {
    fn find_shortest_path(
        &self,
        ctx: &Context,
        params: ShortestPathParams,
        tx: ObserverImpl<PathResult>,
    ) {
        // Optional: Check cache in storage
        if let Some(storage) = &self.storage {
            let cache_key = format!(
                "path:{}:{}:{}",
                ctx.session_id, params.start_idx, params.end_idx
            );
            if let Some(cached_bytes) = storage.get(&cache_key) {
                // Try to deserialize cached result
                if let Ok(cached_result) = serde_json::from_slice::<PathResult>(&cached_bytes) {
                    tx.next(cached_result);
                    tx.complete("Path found (cached)".to_string());
                    return;
                }
            }
        }

        // Compute the shortest path using core logic
        match compute_shortest_path(&params.points, &params.edges, params.start_idx, params.end_idx)
        {
            Ok(result) => {
                // Optional: Cache the result
                if let Some(storage) = &self.storage {
                    let cache_key = format!(
                        "path:{}:{}:{}",
                        ctx.session_id, params.start_idx, params.end_idx
                    );
                    if let Ok(serialized) = serde_json::to_vec(&result) {
                        storage.set(&cache_key, serialized);
                    }
                }

                tx.next(result);
                tx.complete("Path found successfully".to_string());
            }
            Err(error) => {
                tx.error(error);
            }
        }
    }

    fn compute_graph_metrics(
        &self,
        _ctx: &Context,
        params: GraphMetricsParams,
        tx: ObserverImpl<GraphMetrics>,
    ) {
        let node_count = params.points.len();
        let edge_count = params.edges.len();
        
        let total_edge_length: f64 = params.edges.iter()
            .map(|edge| {
                crate::euclidean_distance(&params.points[edge.from], &params.points[edge.to])
            })
            .sum();
        
        let avg_edge_length = if edge_count > 0 {
            total_edge_length / edge_count as f64
        } else {
            0.0
        };

        let metrics = GraphMetrics {
            node_count,
            edge_count,
            total_edge_length,
            avg_edge_length,
        };

        tx.next(metrics);
        tx.complete("Metrics computed successfully".to_string());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use shared_types::storage::InMemoryStorage;
    use shared_types::{Edge, Point};

    // Mock WireResponseSender for testing
    struct MockSender {
        responses: Arc<std::sync::Mutex<Vec<String>>>,
    }

    impl shared_types::router::WireResponseSender for MockSender {
        fn send_response(&self, _wire_response: shared_types::router::WireResponse) {
            // For testing, we could capture responses here
        }
    }

    #[test]
    fn test_handler_without_storage() {
        let handler: PathfinderHandler<InMemoryStorage> = PathfinderHandler::new(None);

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

        let params = ShortestPathParams {
            points,
            edges,
            start_idx: 0,
            end_idx: 2,
        };

        let ctx = Context::new("test-session".to_string(), 1);

        // Create a mock observer
        let sender = Box::new(MockSender {
            responses: Arc::new(std::sync::Mutex::new(Vec::new())),
        });
        let tx = ObserverImpl::new(1, sender);

        // This should not panic
        handler.find_shortest_path(&ctx, params, tx);
    }
}
