use crate::ast::{InputSpec, OptionNode, StreamInfo};
use crate::codec_db::CodecDatabase;
use shared_types::{DiagnosticKind, DiagnosticMessage, Severity, SourceCodeSpan, StreamType, DiagnosticSpan, SpanRole};
use std::collections::HashMap;

/// Track streams through the FFmpeg pipeline
pub struct StreamTracker {
    /// Streams available from inputs
    pub input_streams: Vec<StreamInfo>,
    /// Input file spans by input index
    pub input_file_spans: Vec<SourceCodeSpan>,
    /// Named filter outputs (from filter_complex)
    pub filter_outputs: HashMap<String, StreamType>,
    /// Codec database
    db: CodecDatabase,
}

impl StreamTracker {
    pub fn new() -> Self {
        StreamTracker {
            input_streams: Vec::new(),
            input_file_spans: Vec::new(),
            filter_outputs: HashMap::new(),
            db: CodecDatabase::new(),
        }
    }
    
    /// Analyze inputs and determine available streams
    pub fn analyze_inputs(&mut self, inputs: &[InputSpec]) -> Vec<DiagnosticMessage> {
        let mut diagnostics = Vec::new();
        
        for (input_idx, input) in inputs.iter().enumerate() {
            // Track input file span by index for reference spans
            if self.input_file_spans.len() <= input_idx {
                self.input_file_spans.push(input.file_path_span.clone());
            } else {
                self.input_file_spans[input_idx] = input.file_path_span.clone();
            }
            // Infer stream types from file extension or format options
            let streams = self.infer_input_streams(input);
            
            for (stream_idx, stream_type) in streams.iter().enumerate() {
                self.input_streams.push(StreamInfo {
                    stream_type: stream_type.clone(),
                    index: stream_idx,
                    input_index: input_idx,
                });
            }
            
            // If we couldn't infer any streams, add a warning
            if streams.is_empty() {
                diagnostics.push(DiagnosticMessage {
                    code: "W200".to_string(),
                    severity: Severity::Warning,
                    kind: DiagnosticKind::ParseError {
                        message: "Could not determine stream types from input".to_string(),
                    },
                    message: format!("Unknown stream types for input: {}", input.file_path),
                    spans: vec![DiagnosticSpan { span: input.file_path_span.clone(), role: SpanRole::Target, message: "unknown streams".to_string() }],
                    rich: None,
                });
            }
        }
        
        diagnostics
    }
    
    fn infer_input_streams(&self, input: &InputSpec) -> Vec<StreamType> {
        // Check for explicit format option
        for option in &input.options {
            if let OptionNode::Format { format, .. } = option {
                return self.infer_streams_from_format(format);
            }
        }
        
        // Infer from file extension
        self.infer_streams_from_filename(&input.file_path)
    }
    
    fn infer_streams_from_filename(&self, filename: &str) -> Vec<StreamType> {
        let ext = filename.split('.').last().unwrap_or("");
        
        match ext {
            // Video formats (typically have both video and audio)
            "mp4" | "mkv" | "avi" | "mov" | "webm" | "flv" | "wmv" | "m4v" => {
                vec![StreamType::Video, StreamType::Audio]
            }
            // Audio-only formats
            "mp3" | "aac" | "flac" | "wav" | "ogg" | "opus" | "m4a" | "wma" => {
                vec![StreamType::Audio]
            }
            // Image formats (single video frame)
            "png" | "jpg" | "jpeg" | "bmp" | "gif" => {
                vec![StreamType::Video]
            }
            // Subtitle formats
            "srt" | "ass" | "ssa" | "vtt" => {
                vec![StreamType::Subtitle]
            }
            // Unknown - assume video + audio
            _ => vec![StreamType::Video, StreamType::Audio],
        }
    }
    
    fn infer_streams_from_format(&self, format: &str) -> Vec<StreamType> {
        // Similar logic to filename inference
        self.infer_streams_from_filename(&format!("file.{}", format))
    }
    
    /// Check if a stream type is available in inputs
    pub fn has_stream_type(&self, stream_type: &StreamType) -> bool {
        self.input_streams.iter().any(|s| matches_stream_type(&s.stream_type, stream_type))
    }
    
    /// Get streams of a specific type
    pub fn get_streams_of_type(&self, stream_type: &StreamType) -> Vec<&StreamInfo> {
        self.input_streams
            .iter()
            .filter(|s| matches_stream_type(&s.stream_type, stream_type))
            .collect()
    }
    
    /// Validate filter against available stream types
    pub fn validate_filter(
        &self,
        filter_name: &str,
        expected_type: &StreamType,
        span: &SourceCodeSpan,
    ) -> Option<DiagnosticMessage> {
        if let Some(filter_info) = self.db.get_filter(filter_name) {
            // Check if we have the required input stream type
            if !self.has_stream_type(&filter_info.input_type) {
                // Build spans: target on option span, plus a reference to the first input lacking stream
                let mut spans = vec![DiagnosticSpan { span: span.clone(), role: SpanRole::Target, message: "missing required stream".to_string() }];
                // Find an input index that lacks the required type
                let mut ref_added = false;
                for (idx, input_span) in self.input_file_spans.iter().enumerate() {
                    let has_required = self.input_streams.iter().any(|s|
                        s.input_index == idx && matches_stream_type(&s.stream_type, &filter_info.input_type)
                    );
                    if !has_required {
                        spans.push(DiagnosticSpan { span: input_span.clone(), role: SpanRole::Reference, message: format!("no {:?} stream in input", filter_info.input_type) });
                        ref_added = true;
                        break;
                    }
                }
                if !ref_added {
                    // Fallback: reference the first input if none found (shouldn't happen)
                    if let Some(first) = self.input_file_spans.first() {
                        spans.push(DiagnosticSpan { span: first.clone(), role: SpanRole::Reference, message: format!("no {:?} stream in input", filter_info.input_type) });
                    }
                }
                return Some(DiagnosticMessage {
                    code: "E104".to_string(),
                    severity: Severity::Error,
                    kind: DiagnosticKind::MissingStream {
                        stream_type: filter_info.input_type.clone(),
                        operation: format!("filter '{}'", filter_name),
                    },
                    message: format!(
                        "Filter '{}' requires {:?} stream, but no {:?} stream is available",
                        filter_name, filter_info.input_type, filter_info.input_type
                    ),
                    spans,
                    rich: None,
                });
            }
            
            // Check if filter type matches expected type
            if !matches_stream_type(&filter_info.input_type, expected_type) {
                return Some(DiagnosticMessage {
                    code: "E101".to_string(),
                    severity: Severity::Error,
                    kind: DiagnosticKind::StreamTypeMismatch {
                        filter: filter_name.to_string(),
                        expected: expected_type.clone(),
                        found: filter_info.input_type.clone(),
                    },
                    message: format!(
                        "Filter '{}' expects {:?} stream but is being used in {:?} context",
                        filter_name, filter_info.input_type, expected_type
                    ),
                    spans: vec![DiagnosticSpan { span: span.clone(), role: SpanRole::Target, message: "missing required stream".to_string() }],
                rich: None,                });
            }
        } else {
            // Unknown filter - issue warning
            return Some(DiagnosticMessage {
                code: "E502".to_string(),
                severity: Severity::Warning,
                kind: DiagnosticKind::UnknownFilter {
                    filter: filter_name.to_string(),
                },
                message: format!("Unknown filter: '{}'", filter_name),
                spans: vec![DiagnosticSpan { span: span.clone(), role: SpanRole::Target, message: "unknown filter".to_string() }],
                rich: None,
            });
        }
        
        None
    }
    
    /// Validate codec against stream type
    pub fn validate_codec(
        &self,
        codec_name: &str,
        expected_type: &StreamType,
        span: &SourceCodeSpan,
    ) -> Option<DiagnosticMessage> {
        if codec_name == "copy" {
            // 'copy' is always valid
            return None;
        }
        
        if let Some(codec_info) = self.db.get_codec(codec_name) {
            if !matches_stream_type(&codec_info.stream_type, expected_type) {
                return Some(DiagnosticMessage {
                    code: "E205".to_string(),
                    severity: Severity::Error,
                    kind: DiagnosticKind::InvalidCodecForStream {
                        codec: codec_name.to_string(),
                        stream_type: expected_type.clone(),
                    },
                    message: format!(
                        "Codec '{}' is a {:?} codec but is being used for {:?} stream",
                        codec_name, codec_info.stream_type, expected_type
                    ),
                    spans: vec![DiagnosticSpan { span: span.clone(), role: SpanRole::Target, message: "invalid codec for stream".to_string() }],
                    rich: None,
                });
            }
        } else {
            // Unknown codec - issue warning
            return Some(DiagnosticMessage {
                code: "W201".to_string(),
                severity: Severity::Warning,
                kind: DiagnosticKind::ParseError {
                    message: format!("Unknown codec: '{}'", codec_name),
                },
                message: format!("Unknown codec: '{}'", codec_name),
                spans: vec![DiagnosticSpan { span: span.clone(), role: SpanRole::Target, message: "unknown codec".to_string() }],
                rich: None,
            });
        }
        
        None
    }
    
    /// Validate codec/format compatibility
    pub fn validate_codec_format_compatibility(
        &self,
        codec_name: &str,
        format: &str,
        codec_span: &SourceCodeSpan,
        format_span: &SourceCodeSpan,
    ) -> Option<DiagnosticMessage> {
        if codec_name == "copy" {
            return None;
        }
        
        if !self.db.is_codec_supported_in_format(codec_name, format) {
            return Some(DiagnosticMessage {
                code: "E201".to_string(),
                severity: Severity::Error,
                kind: DiagnosticKind::CodecFormatIncompatible {
                    codec: codec_name.to_string(),
                    format: format.to_string(),
                    reason: format!("Codec '{}' is not supported in '{}' container", codec_name, format),
                },
                message: format!("Codec '{}' is not supported in '{}' container", codec_name, format),
                spans: vec![
                    DiagnosticSpan { span: codec_span.clone(), role: SpanRole::Target, message: "codec".to_string() },
                    DiagnosticSpan { span: format_span.clone(), role: SpanRole::Reference, message: format!("{} container", format) },
                ],
                rich: None,
            });
        }
        
        None
    }
}

fn matches_stream_type(actual: &StreamType, expected: &StreamType) -> bool {
    match (actual, expected) {
        (StreamType::Unknown, _) | (_, StreamType::Unknown) => true,
        (a, b) => std::mem::discriminant(a) == std::mem::discriminant(b),
    }
}

impl Default for StreamTracker {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_infer_video_streams() {
        let tracker = StreamTracker::new();
        let streams = tracker.infer_streams_from_filename("video.mp4");
        assert_eq!(streams.len(), 2);
        assert!(matches!(streams[0], StreamType::Video));
        assert!(matches!(streams[1], StreamType::Audio));
    }
    
    #[test]
    fn test_infer_audio_streams() {
        let tracker = StreamTracker::new();
        let streams = tracker.infer_streams_from_filename("audio.mp3");
        assert_eq!(streams.len(), 1);
        assert!(matches!(streams[0], StreamType::Audio));
    }
    
    #[test]
    fn test_validate_filter() {
        let mut tracker = StreamTracker::new();
        tracker.input_streams.push(StreamInfo {
            stream_type: StreamType::Video,
            index: 0,
            input_index: 0,
        });
        
        let span = SourceCodeSpan {
            start_line: 1,
            start_column: 0,
            end_line: 1,
            end_column: 10,
        };
        
        // Video filter on video stream should be ok
        let result = tracker.validate_filter("scale", &StreamType::Video, &span);
        assert!(result.is_none());
    }
}

