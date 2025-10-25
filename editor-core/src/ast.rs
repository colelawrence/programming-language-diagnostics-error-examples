use shared_types::{SourceCodeSpan, StreamType};

/// Top-level FFmpeg command AST
#[derive(Debug, Clone)]
pub struct FfmpegCommand {
    pub global_options: Vec<OptionNode>,
    pub inputs: Vec<InputSpec>,
    pub outputs: Vec<OutputSpec>,
    pub span: SourceCodeSpan,
}

/// Input file specification with options
#[derive(Debug, Clone)]
pub struct InputSpec {
    pub options: Vec<OptionNode>,
    pub file_path: String,
    pub file_path_span: SourceCodeSpan,
    pub span: SourceCodeSpan,
}

/// Output file specification with options
#[derive(Debug, Clone)]
pub struct OutputSpec {
    pub options: Vec<OptionNode>,
    pub file_path: String,
    pub file_path_span: SourceCodeSpan,
    pub span: SourceCodeSpan,
}

/// An option/flag with its value
#[derive(Debug, Clone)]
pub enum OptionNode {
    // Global options
    GlobalFlag {
        name: String,
        span: SourceCodeSpan,
    },
    
    // Codec options
    VideoCodec {
        codec: String,
        codec_span: SourceCodeSpan,
        span: SourceCodeSpan,
    },
    AudioCodec {
        codec: String,
        codec_span: SourceCodeSpan,
        span: SourceCodeSpan,
    },
    Codec {
        codec: String,
        codec_span: SourceCodeSpan,
        span: SourceCodeSpan,
    },
    
    // Bitrate options
    VideoBitrate {
        bitrate: String,
        bitrate_span: SourceCodeSpan,
        span: SourceCodeSpan,
    },
    AudioBitrate {
        bitrate: String,
        bitrate_span: SourceCodeSpan,
        span: SourceCodeSpan,
    },
    
    // Resolution and frame rate
    Resolution {
        resolution: String,
        resolution_span: SourceCodeSpan,
        span: SourceCodeSpan,
    },
    FrameRate {
        rate: String,
        rate_span: SourceCodeSpan,
        span: SourceCodeSpan,
    },
    
    // Filters
    VideoFilter {
        filter: FilterSpec,
        span: SourceCodeSpan,
    },
    AudioFilter {
        filter: FilterSpec,
        span: SourceCodeSpan,
    },
    FilterComplex {
        filter: FilterSpec,
        span: SourceCodeSpan,
    },
    
    // Stream mapping
    Map {
        mapping: String,
        mapping_span: SourceCodeSpan,
        span: SourceCodeSpan,
    },
    
    // Format
    Format {
        format: String,
        format_span: SourceCodeSpan,
        span: SourceCodeSpan,
    },
    
    // Time options
    SeekStart {
        time: String,
        time_span: SourceCodeSpan,
        span: SourceCodeSpan,
    },
    Duration {
        time: String,
        time_span: SourceCodeSpan,
        span: SourceCodeSpan,
    },
    
    // Audio options
    SampleRate {
        rate: String,
        rate_span: SourceCodeSpan,
        span: SourceCodeSpan,
    },
    AudioChannels {
        channels: String,
        channels_span: SourceCodeSpan,
        span: SourceCodeSpan,
    },
    
    // Generic option (catch-all)
    Generic {
        name: String,
        value: Option<String>,
        value_span: Option<SourceCodeSpan>,
        span: SourceCodeSpan,
    },
}

/// Filter specification
#[derive(Debug, Clone)]
pub struct FilterSpec {
    pub raw: String,
    pub parsed: Option<FilterGraph>,
    pub span: SourceCodeSpan,
}

/// Parsed filter graph (for advanced analysis)
#[derive(Debug, Clone)]
pub struct FilterGraph {
    pub chains: Vec<FilterChain>,
}

#[derive(Debug, Clone)]
pub struct FilterChain {
    pub filters: Vec<Filter>,
}

#[derive(Debug, Clone)]
pub struct Filter {
    pub name: String,
    pub name_span: SourceCodeSpan,
    pub params: Vec<FilterParam>,
    pub span: SourceCodeSpan,
}

#[derive(Debug, Clone)]
pub struct FilterParam {
    pub key: Option<String>,
    pub value: String,
    pub span: SourceCodeSpan,
}

/// Helper function to create SourceCodeSpan from pest::Span with offsets
/// line_offset and column_offset are 1-based positions in the original document
pub fn span_from_pest(span: pest::Span, line_offset: usize, column_offset: usize) -> SourceCodeSpan {
    let (start_line, start_col) = span.start_pos().line_col();
    let (end_line, end_col) = span.end_pos().line_col();
    
    // Pest gives 1-based line/col numbers
    // We want to output 1-based line numbers (for Monaco Editor)
    // For single-line inputs, pest will always report line 1
    // So we replace pest's line with the actual line from line_offset
    
    SourceCodeSpan {
        // Since we're parsing single lines, pest line will be 1
        // Use line_offset as the actual line number (1-based)
        start_line: if start_line == 1 { line_offset } else { start_line - 1 + line_offset },
        start_column: if start_line == 1 { 
            start_col.saturating_sub(1) + column_offset 
        } else { 
            start_col.saturating_sub(1) 
        },
        end_line: if end_line == 1 { line_offset } else { end_line - 1 + line_offset },
        end_column: if end_line == 1 { 
            end_col.saturating_sub(1) + column_offset 
        } else { 
            end_col.saturating_sub(1) 
        },
    }
}

/// Stream information tracked during analysis
#[derive(Debug, Clone)]
pub struct StreamInfo {
    pub stream_type: StreamType,
    pub index: usize,
    pub input_index: usize,
}

