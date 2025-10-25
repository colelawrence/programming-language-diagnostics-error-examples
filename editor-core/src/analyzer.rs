use crate::ast::{FfmpegCommand, OptionNode, OutputSpec};
use crate::codec_db::CodecDatabase;
use crate::stream_tracker::StreamTracker;
use shared_types::{AnalyzerDiagnostics, DiagnosticKind, DiagnosticMessage, Severity, SourceCodeSpan, StreamType, DiagnosticRich, RichBlock, DiagnosticSpan, SpanRole};

/// Analyze FFmpeg command and return diagnostics
pub fn analyze_command(command: FfmpegCommand) -> AnalyzerDiagnostics {
    let mut diagnostics = Vec::new();
    let mut tracker = StreamTracker::new();
    let db = CodecDatabase::new();
    
    // Phase 1: Discover streams from inputs
    let input_diagnostics = tracker.analyze_inputs(&command.inputs);
    diagnostics.extend(input_diagnostics);
    
    // Phase 2: Validate outputs
    for output in &command.outputs {
        let output_diagnostics = analyze_output(output, &tracker, &db);
        diagnostics.extend(output_diagnostics);
    }
    
    AnalyzerDiagnostics { messages: diagnostics }
}

fn analyze_output(
    output: &OutputSpec,
    tracker: &StreamTracker,
    db: &CodecDatabase,
) -> Vec<DiagnosticMessage> {
    let mut diagnostics = Vec::new();
    
    // Infer output format from filename
    let output_format = db.infer_format_from_filename(&output.file_path);
    
    let mut video_codec = None;
    let mut audio_codec = None;
    let mut explicit_format = output_format.clone();
    
    // Collect codec and format information
    for option in &output.options {
        match option {
            OptionNode::VideoCodec { codec, codec_span, .. } => {
                video_codec = Some((codec.clone(), codec_span.clone()));
                
                // Validate that codec is actually a video codec
                if let Some(diag) = tracker.validate_codec(codec, &StreamType::Video, codec_span) {
                    diagnostics.push(diag);
                }
                
                // Check if we have video streams available
                if !tracker.has_stream_type(&StreamType::Video) && codec != "copy" {
                    diagnostics.push(DiagnosticMessage {
                        code: "E104".to_string(),
                        severity: Severity::Error,
                        kind: DiagnosticKind::MissingStream {
                            stream_type: StreamType::Video,
                            operation: "video encoding".to_string(),
                        },
                        message: "Video codec specified but no video stream available in inputs".to_string(),
                        spans: vec![DiagnosticSpan { span: codec_span.clone(), role: SpanRole::Target, message: "filter requires video".to_string() }],
                        rich: None,
                    });
                }
            }
            
            OptionNode::AudioCodec { codec, codec_span, .. } => {
                audio_codec = Some((codec.clone(), codec_span.clone()));
                
                // Validate that codec is actually an audio codec
                if let Some(diag) = tracker.validate_codec(codec, &StreamType::Audio, codec_span) {
                    diagnostics.push(diag);
                }
                
                // Check if we have audio streams available
                if !tracker.has_stream_type(&StreamType::Audio) && codec != "copy" {
                    diagnostics.push(DiagnosticMessage {
                        code: "E105".to_string(),
                        severity: Severity::Error,
                        kind: DiagnosticKind::MissingStream {
                            stream_type: StreamType::Audio,
                            operation: "audio encoding".to_string(),
                        },
                        message: "Audio codec specified but no audio stream available in inputs".to_string(),
                        spans: vec![DiagnosticSpan { span: codec_span.clone(), role: SpanRole::Target, message: "codec requires audio".to_string() }],
                        rich: None,
                    });
                }
            }
            
            OptionNode::Codec { codec, codec_span, .. } => {
                // Generic codec - could be video or audio, check both
                if let Some(codec_info) = db.get_codec(codec) {
                    match codec_info.stream_type {
                        StreamType::Video => {
                            video_codec = Some((codec.clone(), codec_span.clone()));
                        }
                        StreamType::Audio => {
                            audio_codec = Some((codec.clone(), codec_span.clone()));
                        }
                        _ => {}
                    }
                }
            }
            
            OptionNode::Format { format, format_span: _format_span, .. } => {
                explicit_format = Some(format.clone());
            }
            
            OptionNode::VideoFilter { filter, span } => {
                // Parse filter name from raw filter string
                let filter_name = extract_filter_name(&filter.raw);
                if let Some(mut diag) = tracker.validate_filter(&filter_name, &StreamType::Video, span) {
                    // Attach a sample Mermaid diagram for type mismatch errors
                    if matches!(diag.kind, DiagnosticKind::StreamTypeMismatch{..}) {
                        diag.rich = Some(DiagnosticRich { blocks: vec![
                            RichBlock::MarkdownGfm { markdown: format!("Filter '{}' expects video input.", filter_name) },
                            RichBlock::Mermaid { mermaid: "graph TD; in_audio([audio]) --x--> vf_scale[scale]; vf_scale --x--> out([video])".to_string() }
                        ]});
                    } else {
                        diag.rich = None;
                    }
                    diagnostics.push(diag);
                }
            }
            
            OptionNode::AudioFilter { filter, span } => {
                let filter_name = extract_filter_name(&filter.raw);
                if let Some(diag) = tracker.validate_filter(&filter_name, &StreamType::Audio, span) {
                    diagnostics.push(diag);
                }
            }
            
            OptionNode::Resolution { resolution, resolution_span, .. } => {
                if let Some(diag) = validate_resolution(resolution, resolution_span) {
                    diagnostics.push(diag);
                }
                
                // Check for upscaling (would need input resolution info)
                // TODO: Implement resolution tracking and upscaling detection
            }
            
            OptionNode::VideoBitrate { bitrate, bitrate_span, .. } => {
                if let Some(diag) = validate_bitrate(bitrate, bitrate_span, true) {
                    diagnostics.push(diag);
                }
            }
            
            OptionNode::AudioBitrate { bitrate, bitrate_span, .. } => {
                if let Some(diag) = validate_bitrate(bitrate, bitrate_span, false) {
                    diagnostics.push(diag);
                }
            }
            
            OptionNode::FrameRate { rate, rate_span, .. } => {
                if let Some(diag) = validate_framerate(rate, rate_span) {
                    diagnostics.push(diag);
                }
            }
            
            OptionNode::Map { mapping, mapping_span, .. } => {
                // Validate stream mapping
                if let Some(diag) = validate_mapping(mapping, mapping_span, tracker) {
                    diagnostics.push(diag);
                }
            }
            
            _ => {}
        }
    }
    
    // Phase 3: Check codec/format compatibility
    if let Some(format) = &explicit_format {
        if let Some((codec, codec_span)) = &video_codec {
            if let Some(diag) = tracker.validate_codec_format_compatibility(
                codec,
                format,
                codec_span,
                &output.file_path_span,
            ) {
                diagnostics.push(diag);
            }
        }
        
        if let Some((codec, codec_span)) = &audio_codec {
            if let Some(diag) = tracker.validate_codec_format_compatibility(
                codec,
                format,
                codec_span,
                &output.file_path_span,
            ) {
                diagnostics.push(diag);
            }
        }
    }
    
    diagnostics
}

fn extract_filter_name(filter_str: &str) -> String {
    // Extract first filter name from filter string (before '=' or ',')
    filter_str
        .split(&['=', ',', ':'][..])
        .next()
        .unwrap_or("")
        .trim()
        .to_string()
}

fn validate_resolution(resolution: &str, span: &SourceCodeSpan) -> Option<DiagnosticMessage> {
    // Check format: NxM where N and M are numbers
    let parts: Vec<&str> = resolution.split('x').collect();
    if parts.len() != 2 {
        return Some(DiagnosticMessage {
            code: "E401".to_string(),
            severity: Severity::Error,
            kind: DiagnosticKind::InvalidResolution {
                value: resolution.to_string(),
            },
            message: format!("Invalid resolution format '{}' (expected format: WIDTHxHEIGHT)", resolution),
            spans: vec![DiagnosticSpan { span: span.clone(), role: SpanRole::Target, message: "invalid resolution format".to_string() }],
            rich: None,
        });
    }
    
    // Validate both parts are numbers
    if parts[0].parse::<u32>().is_err() || parts[1].parse::<u32>().is_err() {
        return Some(DiagnosticMessage {
            code: "E401".to_string(),
            severity: Severity::Error,
            kind: DiagnosticKind::InvalidResolution {
                value: resolution.to_string(),
            },
            message: format!("Invalid resolution '{}' (width and height must be numbers)", resolution),
            spans: vec![DiagnosticSpan { span: span.clone(), role: SpanRole::Target, message: "width/height must be numbers".to_string() }],
            rich: None,
        });
    }
    
    None
}

fn validate_bitrate(bitrate: &str, span: &SourceCodeSpan, is_video: bool) -> Option<DiagnosticMessage> {
    // Extract numeric part
    let numeric_part = bitrate.trim_end_matches(|c: char| c.is_alphabetic());
    
    if let Ok(value) = numeric_part.parse::<u32>() {
        // Check for extremely high bitrates (warning)
        let threshold = if is_video { 50000 } else { 500 }; // 50Mbps for video, 500kbps for audio
        
        if value > threshold {
            return Some(DiagnosticMessage {
                code: "W101".to_string(),
                severity: Severity::Warning,
                kind: DiagnosticKind::HighBitrateWarning {
                    bitrate: bitrate.to_string(),
                },
                message: format!("Extremely high bitrate specified: {}", bitrate),
                spans: vec![DiagnosticSpan { span: span.clone(), role: SpanRole::Target, message: "high bitrate".to_string() }],
                rich: None,
            });
        }
    } else {
        return Some(DiagnosticMessage {
            code: "E402".to_string(),
            severity: Severity::Error,
            kind: DiagnosticKind::InvalidBitrate {
                value: bitrate.to_string(),
            },
            message: format!("Invalid bitrate format '{}'", bitrate),
            spans: vec![DiagnosticSpan { span: span.clone(), role: SpanRole::Target, message: "invalid bitrate".to_string() }],
            rich: None,
        });
    }
    
    None
}

fn validate_framerate(rate: &str, span: &SourceCodeSpan) -> Option<DiagnosticMessage> {
    if let Ok(fps) = rate.parse::<f64>() {
        if fps <= 0.0 || fps > 1000.0 {
            return Some(DiagnosticMessage {
                code: "E403".to_string(),
                severity: Severity::Error,
                kind: DiagnosticKind::InvalidFrameRate {
                    value: rate.to_string(),
                },
                message: format!("Invalid frame rate '{}' (must be between 0 and 1000)", rate),
                spans: vec![DiagnosticSpan { span: span.clone(), role: SpanRole::Target, message: "invalid frame rate".to_string() }],
                rich: None,
            });
        }
    } else {
        return Some(DiagnosticMessage {
            code: "E403".to_string(),
            severity: Severity::Error,
            kind: DiagnosticKind::InvalidFrameRate {
                value: rate.to_string(),
            },
            message: format!("Invalid frame rate format '{}'", rate),
            spans: vec![DiagnosticSpan { span: span.clone(), role: SpanRole::Target, message: "invalid frame rate format".to_string() }],
            rich: None,
        });
    }
    
    None
}

fn validate_mapping(
    mapping: &str,
    span: &SourceCodeSpan,
    tracker: &StreamTracker,
) -> Option<DiagnosticMessage> {
    // Parse mapping format: [input_index]:[stream_type]:[stream_index] or [label]
    
    if mapping.starts_with('[') && mapping.ends_with(']') {
        // Filter label reference
        let label = &mapping[1..mapping.len()-1];
        if !tracker.filter_outputs.contains_key(label) {
            return Some(DiagnosticMessage {
                code: "E303".to_string(),
                severity: Severity::Error,
                kind: DiagnosticKind::StreamMappingError {
                    mapping: mapping.to_string(),
                    reason: format!("Filter output label '{}' does not exist", label),
                },
                message: format!("Referenced filter output '{}' does not exist", label),
                spans: vec![DiagnosticSpan { span: span.clone(), role: SpanRole::Target, message: "unknown label".to_string() }],
                rich: None,
            });
        }
    } else {
        // Stream index reference
        let parts: Vec<&str> = mapping.split(':').collect();
        
        if let Some(input_idx_str) = parts.first() {
            if let Ok(input_idx) = input_idx_str.parse::<usize>() {
                // Check if input exists
                let max_input = tracker.input_streams
                    .iter()
                    .map(|s| s.input_index)
                    .max()
                    .unwrap_or(0);
                
                if input_idx > max_input {
                    return Some(DiagnosticMessage {
                        code: "E301".to_string(),
                        severity: Severity::Error,
                        kind: DiagnosticKind::NonExistentStream {
                            stream_ref: mapping.to_string(),
                        },
                        message: format!("Input index {} does not exist", input_idx),
                        spans: vec![DiagnosticSpan { span: span.clone(), role: SpanRole::Target, message: "non-existent input index".to_string() }],
                        rich: None,
                    });
                }
            }
        }
    }
    
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::parse_command;
    
    #[test]
    fn test_analyze_simple_command() {
        let input = "ffmpeg -i input.mp4 output.mp4";
        let cmd = parse_command(input, 0, 0).unwrap();
        let result = analyze_command(cmd);
        // Should have no errors for simple valid command
        assert!(result.messages.is_empty() || result.messages.iter().all(|m| matches!(m.severity, Severity::Warning | Severity::Info)));
    }
    
    #[test]
    fn test_detect_video_codec_on_audio() {
        let input = "ffmpeg -i audio.mp3 -c:v libx264 output.mp4";
        let cmd = parse_command(input, 0, 0).unwrap();
        let result = analyze_command(cmd);
        // Should detect that we're trying to use video codec on audio-only input
        let has_error = result.messages.iter().any(|m| 
            matches!(m.severity, Severity::Error) && m.code == "E104"
        );
        assert!(has_error);
    }
    
    #[test]
    fn test_detect_invalid_resolution() {
        let input = "ffmpeg -i input.mp4 -s 1920 output.mp4";
        let cmd = parse_command(input, 0, 0).unwrap();
        let result = analyze_command(cmd);
        // Should detect invalid resolution format
        let has_error = result.messages.iter().any(|m| m.code == "E401");
        assert!(has_error);
    }
}

