use crate::parser::parse_command;
use crate::analyzer::analyze_command;
use shared_types::context::Context;
use shared_types::router::{CallHandler, ObserverImpl};
use shared_types::storage::Storage;
use shared_types::{
    AnalyzeCodeParams, AnalyzerDiagnostics, DiagnosticKind, DiagnosticMessage, Severity,
    SourceCodeSpan, DiagnosticSpan, SpanRole,
};
use std::sync::Arc;

/// EditorHandler implements the CallHandler trait for FFmpeg command analysis
/// This is the transport-agnostic business logic handler
pub struct EditorHandler<S: Storage> {
    storage: Option<Arc<S>>,
}

impl<S: Storage> EditorHandler<S> {
    pub fn new(storage: Option<Arc<S>>) -> Self {
        Self { storage }
    }
}

impl<S: Storage> CallHandler for EditorHandler<S> {
    fn analyze_code(
        &self,
        _ctx: &Context,
        params: AnalyzeCodeParams,
        tx: ObserverImpl<AnalyzerDiagnostics>,
    ) {
        // Parse and analyze FFmpeg command with offsets
        let analysis_result = analyze_content(&params.content, params.line_offset, params.column_offset);

        tx.next(analysis_result);
        tx.complete("Analysis complete".to_string());
    }

    // Keep the pathfinder methods for now (we'll remove them during cleanup)
    fn find_shortest_path(
        &self,
        _ctx: &Context,
        _params: shared_types::ShortestPathParams,
        tx: ObserverImpl<shared_types::PathResult>,
    ) {
        tx.error("Pathfinding not supported in editor mode".to_string());
    }

    fn compute_graph_metrics(
        &self,
        _ctx: &Context,
        _params: shared_types::GraphMetricsParams,
        tx: ObserverImpl<shared_types::GraphMetrics>,
    ) {
        tx.error("Graph metrics not supported in editor mode".to_string());
    }
}

/// Analyze FFmpeg command and return diagnostics with offset support
fn analyze_content(content: &str, line_offset: usize, column_offset: usize) -> AnalyzerDiagnostics {
    // Parse the FFmpeg command with offsets
    match parse_command(content, line_offset, column_offset) {
        Ok(command) => {
            // Run semantic analysis
            analyze_command(command)
        }
        Err(parse_error) => {
            // Return parse error as diagnostic
            AnalyzerDiagnostics {
                messages: vec![DiagnosticMessage {
                    code: "E000".to_string(),
                    severity: Severity::Error,
                    kind: DiagnosticKind::ParseError {
                        message: parse_error.clone(),
                    },
                    message: format!("Failed to parse FFmpeg command: {}", parse_error),
                    spans: vec![DiagnosticSpan {
                        span: SourceCodeSpan {
                            start_line: line_offset,
                            start_column: column_offset,
                            end_line: line_offset,
                            end_column: column_offset + content.len().min(100),
                        },
                        role: SpanRole::Target,
                        message: "parse error here".to_string(),
                    }],
                    rich: None,
                }],
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use shared_types::storage::InMemoryStorage;

    #[test]
    fn test_analyze_valid_command() {
        let result = analyze_content("ffmpeg -i input.mp4 output.mp4", 0, 0);
        // Valid command should have no errors (may have warnings)
        let has_errors = result.messages.iter().any(|m| matches!(m.severity, Severity::Error));
        assert!(!has_errors);
    }

    #[test]
    fn test_analyze_video_codec_on_audio() {
        let result = analyze_content("ffmpeg -i audio.mp3 -c:v libx264 output.mp4", 0, 0);
        // Should detect video codec on audio-only input
        let has_error = result.messages.iter().any(|m| m.code == "E104");
        assert!(has_error);
    }

    #[test]
    fn test_analyze_invalid_resolution() {
        let result = analyze_content("ffmpeg -i input.mp4 -s 1920 output.mp4", 0, 0);
        // Should detect invalid resolution format
        let has_error = result.messages.iter().any(|m| m.code == "E401");
        assert!(has_error);
    }

    #[test]
    fn test_analyze_codec_format_incompatibility() {
        let result = analyze_content("ffmpeg -i input.mp4 -c:v vp9 output.mp4", 0, 0);
        // VP9 is not compatible with MP4 container
        let has_error = result.messages.iter().any(|m| m.code == "E201");
        assert!(has_error);
    }

    #[test]
    fn test_handler() {
        let handler: EditorHandler<InMemoryStorage> = EditorHandler::new(None);
        let ctx = Context::new("test-session".to_string(), 1);

        // Mock sender for testing
        struct MockSender;
        impl shared_types::router::WireResponseSender for MockSender {
            fn send_response(&self, _response: shared_types::router::WireResponse) {}
        }

        let tx = ObserverImpl::new(1, Box::new(MockSender));
        let params = AnalyzeCodeParams {
            content: "ffmpeg -i input.mp4 output.mp4".to_string(),
            file_path: None,
            line_offset: 0,
            column_offset: 0,
        };

        handler.analyze_code(&ctx, params, tx);
        // If we get here without panicking, the handler works
    }
}

