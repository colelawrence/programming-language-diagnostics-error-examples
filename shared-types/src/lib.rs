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

// ========== Editor Protocol Types ==========

/// A span of source code defined by line and column positions
#[protocol("wasm")]
pub struct SourceCodeSpan {
    pub start_line: usize,
    pub start_column: usize,
    pub end_line: usize,
    pub end_column: usize,
}

/// Severity level of a diagnostic message
#[protocol("wasm")]
pub enum Severity {
    Error,
    Warning,
    Info,
    Hint,
}

/// Stream type in FFmpeg
#[protocol("wasm")]
#[derive(PartialEq, Eq)]
pub enum StreamType {
    Video,
    Audio,
    Subtitle,
    Data,
    Unknown,
}

/// Role attached to a diagnostic span
#[protocol("wasm")]
pub enum SpanRole {
    Target,
    Reference,
    Suggestion { replacement: String },
}

/// A diagnostic span with role and per-span message
#[protocol("wasm")]
pub struct DiagnosticSpan {
    pub span: SourceCodeSpan,
    pub role: SpanRole,
    pub message: String,
}

/// Rich content blocks for diagnostics (GFM markdown and Mermaid diagrams)
#[protocol("wasm")]
pub enum RichBlock {
    MarkdownGfm { markdown: String },
    Mermaid { mermaid: String },
}

/// Optional rich content attached to a diagnostic
#[protocol("wasm")]
pub struct DiagnosticRich {
    pub blocks: Vec<RichBlock>,
}

/// Discriminated union of all FFmpeg diagnostic kinds
#[protocol("wasm")]
pub enum DiagnosticKind {
    // E100-E199: Stream Type Mismatches
    StreamTypeMismatch { filter: String, expected: StreamType, found: StreamType },
    MissingStream { stream_type: StreamType, operation: String },
    VideoFilterOnAudio { filter: String },
    AudioFilterOnVideo { filter: String },
    
    // E200-E299: Codec/Format Incompatibilities
    CodecFormatIncompatible { codec: String, format: String, reason: String },
    InvalidCodecForStream { codec: String, stream_type: StreamType },
    UnsupportedPixelFormat { format: String, codec: String },
    UnsupportedSampleRate { rate: String, codec: String },
    
    // E300-E399: Stream Mapping Errors
    StreamMappingError { mapping: String, reason: String },
    NonExistentStream { stream_ref: String },
    DuplicateMapping { stream_ref: String },
    AmbiguousStreamSelection { reason: String },
    
    // E400-E499: Parameter/Option Errors
    InvalidParameter { option: String, value: String, reason: String },
    InvalidResolution { value: String },
    InvalidBitrate { value: String },
    InvalidFrameRate { value: String },
    MutuallyExclusiveOptions { option1: String, option2: String },
    MissingRequiredOption { option: String, context: String },
    ParameterOutOfRange { option: String, value: String, min: String, max: String },
    
    // E500-E599: Filter Syntax Errors
    FilterSyntaxError { filter: String, message: String },
    UnknownFilter { filter: String },
    MissingFilterParameter { filter: String, parameter: String },
    InvalidFilterParameter { filter: String, parameter: String, value: String },
    FilterChainTypeMismatch { from_type: StreamType, to_type: StreamType },
    
    // W100-W199: Performance/Quality Warnings
    HighBitrateWarning { bitrate: String },
    ResolutionUpscaling { from_res: String, to_res: String },
    LossyTranscoding { message: String },
    NoQualitySetting { codec: String },
    
    // General errors
    ParseError { message: String },
    UnknownOption { option: String },
}

/// A diagnostic message with its associated source locations
/// This is the primary entity - users see a list of these
#[protocol("wasm")]
pub struct DiagnosticMessage {
    /// Diagnostic code (e.g., "E001", "W002")
    pub code: String,
    /// Severity level
    pub severity: Severity,
    /// The specific kind of diagnostic
    pub kind: DiagnosticKind,
    /// Human-readable message
    pub message: String,
    /// Source code spans where this diagnostic applies, with roles
    /// Multiple spans for diagnostics that reference multiple locations
    pub spans: Vec<DiagnosticSpan>,
    /// Optional rich content (markdown, mermaid diagrams)
    pub rich: Option<DiagnosticRich>,
}

/// Complete response containing all diagnostic messages
#[protocol("wasm")]
pub struct AnalyzerDiagnostics {
    pub messages: Vec<DiagnosticMessage>,
}

/// Parameters for code analysis
#[protocol("wasm")]
#[codegen(fn = "analyze_code() -> AnalyzerDiagnostics")]
pub struct AnalyzeCodeParams {
    /// The source code to analyze
    pub content: String,
    /// Optional file path for context
    pub file_path: Option<String>,
    /// Line offset for error reporting (0-based)
    pub line_offset: usize,
    /// Column offset for error reporting (0-based)
    pub column_offset: usize,
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
