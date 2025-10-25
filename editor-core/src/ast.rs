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

/// Helper to create spans
impl SourceCodeSpan {
    pub fn from_pest_span(span: pest::Span, line_offset: usize) -> Self {
        let (start_line, start_col) = span.start_pos().line_col();
        let (end_line, end_col) = span.end_pos().line_col();
        
        SourceCodeSpan {
            start_line: start_line + line_offset,
            start_column: start_col.saturating_sub(1),
            end_line: end_line + line_offset,
            end_column: end_col.saturating_sub(1),
        }
    }
}

/// Stream information tracked during analysis
#[derive(Debug, Clone)]
pub struct StreamInfo {
    pub stream_type: StreamType,
    pub index: usize,
    pub input_index: usize,
}

