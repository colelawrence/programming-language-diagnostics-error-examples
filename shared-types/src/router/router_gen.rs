use super::{ObserverImpl, WireResponseSender};
use crate::context::Context;
use crate::*;
use serde::{Deserialize, Serialize};

pub trait CallHandler {
    fn find_shortest_path(
        &self,
        ctx: &Context,
        params: ShortestPathParams,
        tx: ObserverImpl<PathResult>,
    );
    fn compute_graph_metrics(
        &self,
        ctx: &Context,
        params: GraphMetricsParams,
        tx: ObserverImpl<GraphMetrics>,
    );
    fn analyze_code(
        &self,
        ctx: &Context,
        params: AnalyzeCodeParams,
        tx: ObserverImpl<AnalyzerDiagnostics>,
    );
}

#[allow(non_camel_case_types)]
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum CallGen {
    find_shortest_path(ShortestPathParams),
    compute_graph_metrics(GraphMetricsParams),
    analyze_code(AnalyzeCodeParams),
}

#[allow(non_camel_case_types)]
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum ResponseNextGen {
    find_shortest_path(PathResult),
    compute_graph_metrics(GraphMetrics),
    analyze_code(AnalyzerDiagnostics),
}

pub(crate) fn gen_call(
    ctx: &Context,
    id: usize,
    call: CallGen,
    handler: &dyn CallHandler,
    sender: Box<dyn WireResponseSender>,
) {
    match call {
        CallGen::find_shortest_path(params) => handler.find_shortest_path(
            ctx,
            params,
            ObserverImpl::new(id, sender),
        ),
        CallGen::compute_graph_metrics(params) => handler.compute_graph_metrics(
            ctx,
            params,
            ObserverImpl::new(id, sender),
        ),
        CallGen::analyze_code(params) => handler.analyze_code(
            ctx,
            params,
            ObserverImpl::new(id, sender),
        ),
    }
}

// ToResponseNextGen implementations
impl super::ToResponseNextGen for PathResult {
    fn to_response_next_gen(self) -> ResponseNextGen {
        ResponseNextGen::find_shortest_path(self)
    }
}

impl super::ToResponseNextGen for GraphMetrics {
    fn to_response_next_gen(self) -> ResponseNextGen {
        ResponseNextGen::compute_graph_metrics(self)
    }
}

impl super::ToResponseNextGen for AnalyzerDiagnostics {
    fn to_response_next_gen(self) -> ResponseNextGen {
        ResponseNextGen::analyze_code(self)
    }
}
