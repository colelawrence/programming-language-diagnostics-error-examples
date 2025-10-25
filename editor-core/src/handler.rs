use shared_types::context::Context;
use shared_types::router::{CallHandler, ObserverImpl};
use shared_types::storage::Storage;
use shared_types::{
    AnalyzeCodeParams, AnalyzerDiagnostics, DiagnosticKind, DiagnosticMessage, Severity,
    SourceCodeSpan,
};
use std::sync::Arc;

/// EditorHandler implements the CallHandler trait for language analysis
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
        // Simple lexical analysis to demonstrate the error protocol
        let analysis_result = analyze_content(&params.content);

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

/// Analyze code content and return diagnostics
fn analyze_content(content: &str) -> AnalyzerDiagnostics {
    let mut messages = Vec::new();

    // Simple demonstration: check for common patterns
    for (line_idx, line) in content.lines().enumerate() {
        let line_num = line_idx + 1;

        // Check for undefined variables (simplified: look for 'undefined' keyword)
        if let Some(col) = line.find("undefined") {
            messages.push(DiagnosticMessage {
                code: "E001".to_string(),
                severity: Severity::Error,
                kind: DiagnosticKind::UndefinedVariable {
                    name: "undefined".to_string(),
                },
                message: "Use of undefined variable".to_string(),
                spans: vec![SourceCodeSpan {
                    start_line: line_num,
                    start_column: col,
                    end_line: line_num,
                    end_column: col + 9, // "undefined".len()
                }],
            });
        }

        // Check for TODO comments
        if let Some(col) = line.find("TODO") {
            messages.push(DiagnosticMessage {
                code: "W001".to_string(),
                severity: Severity::Warning,
                kind: DiagnosticKind::InvalidOperation {
                    operation: "TODO comment".to_string(),
                    reason: "Consider implementing this".to_string(),
                },
                message: "TODO comment found".to_string(),
                spans: vec![SourceCodeSpan {
                    start_line: line_num,
                    start_column: col,
                    end_line: line_num,
                    end_column: line.len(),
                }],
            });
        }

        // Check for duplicate 'let' declarations (very simplified)
        if line.contains("let ") && line.matches("let ").count() > 1 {
            messages.push(DiagnosticMessage {
                code: "E002".to_string(),
                severity: Severity::Error,
                kind: DiagnosticKind::DuplicateDefinition {
                    name: "variable".to_string(),
                },
                message: "Multiple 'let' declarations on one line".to_string(),
                spans: vec![SourceCodeSpan {
                    start_line: line_num,
                    start_column: 0,
                    end_line: line_num,
                    end_column: line.len(),
                }],
            });
        }

        // Check for type mismatches (look for '=' followed by different types)
        if line.contains("String") && line.contains("= 123") {
            if let Some(col) = line.find("= 123") {
                messages.push(DiagnosticMessage {
                    code: "E003".to_string(),
                    severity: Severity::Error,
                    kind: DiagnosticKind::TypeError {
                        expected: "String".to_string(),
                        found: "Number".to_string(),
                    },
                    message: "Type mismatch: expected String, found Number".to_string(),
                    spans: vec![SourceCodeSpan {
                        start_line: line_num,
                        start_column: col,
                        end_line: line_num,
                        end_column: col + 5,
                    }],
                });
            }
        }

        // Check for syntax errors (unclosed braces, etc.)
        let open_braces = line.matches('{').count();
        let close_braces = line.matches('}').count();
        if open_braces != close_braces {
            messages.push(DiagnosticMessage {
                code: "E004".to_string(),
                severity: Severity::Error,
                kind: DiagnosticKind::SyntaxError {
                    message: "Unbalanced braces".to_string(),
                },
                message: format!(
                    "Unbalanced braces: {} open, {} close",
                    open_braces, close_braces
                ),
                spans: vec![SourceCodeSpan {
                    start_line: line_num,
                    start_column: 0,
                    end_line: line_num,
                    end_column: line.len(),
                }],
            });
        }
    }

    AnalyzerDiagnostics { messages }
}

#[cfg(test)]
mod tests {
    use super::*;
    use shared_types::storage::InMemoryStorage;

    #[test]
    fn test_analyze_empty_content() {
        let result = analyze_content("");
        assert_eq!(result.messages.len(), 0);
    }

    #[test]
    fn test_analyze_undefined_variable() {
        let result = analyze_content("let x = undefined;");
        assert_eq!(result.messages.len(), 1);
        assert_eq!(result.messages[0].code, "E001");
        assert!(matches!(result.messages[0].severity, Severity::Error));
    }

    #[test]
    fn test_analyze_todo_comment() {
        let result = analyze_content("// TODO: implement this");
        assert_eq!(result.messages.len(), 1);
        assert_eq!(result.messages[0].code, "W001");
        assert!(matches!(result.messages[0].severity, Severity::Warning));
    }

    #[test]
    fn test_analyze_multiple_errors() {
        let content = "let x = undefined;\n// TODO: fix this\nlet y = 123;";
        let result = analyze_content(content);
        assert!(result.messages.len() >= 2);
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
            content: "let x = undefined;".to_string(),
            file_path: None,
        };

        handler.analyze_code(&ctx, params, tx);
        // If we get here without panicking, the handler works
    }
}

