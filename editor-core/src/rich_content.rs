use crate::ast::{FfmpegCommand, OptionNode};
use crate::codec_db::CodecDatabase;
use crate::stream_tracker::StreamTracker;
use shared_types::{DiagnosticRich, RichBlock, StreamType};

/// Generate pipeline flow diagram (Mermaid) showing data flow through FFmpeg command
pub fn generate_pipeline_diagram(
    command: &FfmpegCommand,
    tracker: &StreamTracker,
    db: &CodecDatabase,
) -> String {
    let mut mermaid = String::from("graph LR\n");
    let mut node_id = 0;
    
    // Generate input nodes
    let mut input_nodes = Vec::new();
    for input in &command.inputs {
        let streams = tracker.get_streams_for_input(&input.file_path);
        let stream_desc = format_stream_types(&streams);
        let input_id = format!("I{}", node_id);
        node_id += 1;
        
        mermaid.push_str(&format!("  {}[{}{}]\n", 
            input_id,
            sanitize_label(&input.file_path),
            if stream_desc.is_empty() { String::new() } else { format!("<br/>{}", stream_desc) }
        ));
        input_nodes.push((input_id, streams));
    }
    
    // Generate codec/processing nodes and output nodes
    for output in &command.outputs {
        let output_id = format!("O{}", node_id);
        node_id += 1;
        
        // Detect codecs and filters
        let mut video_codec = None;
        let mut audio_codec = None;
        let mut filters = Vec::new();
        
        for option in &output.options {
            match option {
                OptionNode::VideoCodec { codec, .. } => video_codec = Some(codec.clone()),
                OptionNode::AudioCodec { codec, .. } => audio_codec = Some(codec.clone()),
                OptionNode::Codec { codec, .. } => {
                    if let Some(info) = db.get_codec(codec) {
                        match info.stream_type {
                            StreamType::Video => video_codec = Some(codec.clone()),
                            StreamType::Audio => audio_codec = Some(codec.clone()),
                            _ => {}
                        }
                    }
                }
                OptionNode::VideoFilter { filter, .. } => filters.push(("video", filter.raw.clone())),
                _ => {}
            }
        }
        
        // Create intermediate nodes for codecs/filters
        let mut last_video_node = None;
        let mut last_audio_node = None;
        
        // Connect inputs to processing nodes
        for (input_id, streams) in &input_nodes {
            if streams.contains(&StreamType::Video) {
                if let Some(ref codec) = video_codec {
                    let vcodec_id = format!("VC{}", node_id);
                    node_id += 1;
                    mermaid.push_str(&format!("  {}[{}]\n", vcodec_id, sanitize_label(codec)));
                    mermaid.push_str(&format!("  {} -->|video| {}\n", input_id, vcodec_id));
                    last_video_node = Some(vcodec_id);
                } else {
                    last_video_node = Some(input_id.clone());
                }
            }
            
            if streams.contains(&StreamType::Audio) {
                if let Some(ref codec) = audio_codec {
                    let acodec_id = format!("AC{}", node_id);
                    node_id += 1;
                    mermaid.push_str(&format!("  {}[{}]\n", acodec_id, sanitize_label(codec)));
                    mermaid.push_str(&format!("  {} -->|audio| {}\n", input_id, acodec_id));
                    last_audio_node = Some(acodec_id);
                } else {
                    last_audio_node = Some(input_id.clone());
                }
            }
        }
        
        // Add filter nodes
        for (filter_type, filter_name) in &filters {
            let filter_id = format!("F{}", node_id);
            node_id += 1;
            mermaid.push_str(&format!("  {}[{}]\n", filter_id, sanitize_label(filter_name)));
            
            if *filter_type == "video" {
                if let Some(prev) = last_video_node.take() {
                    mermaid.push_str(&format!("  {} --> {}\n", prev, filter_id));
                    last_video_node = Some(filter_id);
                }
            }
        }
        
        // Create output node
        let output_format = db.infer_format_from_filename(&output.file_path);
        mermaid.push_str(&format!("  {}[{}{}]\n",
            output_id,
            sanitize_label(&output.file_path),
            if let Some(fmt) = output_format { format!("<br/>{}", fmt) } else { String::new() }
        ));
        
        // Connect to output
        if let Some(vid_node) = last_video_node {
            mermaid.push_str(&format!("  {} --> {}\n", vid_node, output_id));
        }
        if let Some(aud_node) = last_audio_node {
            mermaid.push_str(&format!("  {} --> {}\n", aud_node, output_id));
        }
    }
    
    mermaid
}

/// Generate codec compatibility matrix showing which containers support a given codec
pub fn generate_codec_compatibility_matrix(
    codec_name: &str,
    codec_type: &StreamType,
    attempted_format: Option<&str>,
) -> String {
    let mut mermaid = String::from("graph TD\n");
    
    // Common codec-container compatibility rules
    let (compatible, incompatible) = match (codec_name, codec_type) {
        ("vp9", StreamType::Video) => {
            (vec!["WebM", "MKV"], vec!["MP4", "AVI"])
        }
        ("vp8", StreamType::Video) => {
            (vec!["WebM", "MKV"], vec!["MP4", "AVI"])
        }
        ("av1", StreamType::Video) => {
            (vec!["WebM", "MKV", "MP4"], vec!["AVI"])
        }
        ("libx264" | "h264", StreamType::Video) => {
            (vec!["MP4", "MKV", "AVI", "MOV"], vec!["WebM"])
        }
        ("libx265" | "hevc", StreamType::Video) => {
            (vec!["MP4", "MKV", "MOV"], vec!["WebM", "AVI"])
        }
        ("opus", StreamType::Audio) => {
            (vec!["WebM", "MKV", "OGG"], vec!["MP4", "MP3"])
        }
        ("vorbis", StreamType::Audio) => {
            (vec!["OGG", "WebM", "MKV"], vec!["MP4", "MP3"])
        }
        ("aac", StreamType::Audio) => {
            (vec!["MP4", "MKV", "MOV"], vec!["WebM", "OGG"])
        }
        _ => (vec![], vec![]),
    };
    
    mermaid.push_str(&format!("  Codec[{}]\n", sanitize_label(codec_name)));
    
    for fmt in &compatible {
        let node_id = format!("C{}", fmt.replace(".", ""));
        mermaid.push_str(&format!("  {}[✓ {}]\n", node_id, fmt));
        mermaid.push_str(&format!("  Codec --> {}\n", node_id));
        mermaid.push_str(&format!("  style {} fill:#2a4,stroke:#6f6\n", node_id));
    }
    
    for fmt in &incompatible {
        let node_id = format!("I{}", fmt.replace(".", ""));
        mermaid.push_str(&format!("  {}[✗ {}]\n", node_id, fmt));
        mermaid.push_str(&format!("  Codec -.-> {}\n", node_id));
        
        // Highlight the attempted format in red
        if let Some(attempted) = attempted_format {
            if attempted.eq_ignore_ascii_case(fmt) {
                mermaid.push_str(&format!("  style {} fill:#a22,stroke:#f66\n", node_id));
            } else {
                mermaid.push_str(&format!("  style {} fill:#444,stroke:#888\n", node_id));
            }
        } else {
            mermaid.push_str(&format!("  style {} fill:#444,stroke:#888\n", node_id));
        }
    }
    
    mermaid
}

/// Generate markdown explanation for codec/container incompatibility
pub fn explain_codec_format_incompatibility(
    codec_name: &str,
    format_name: &str,
    compatible_formats: &[&str],
) -> String {
    format!(
        "## Codec/Container Incompatibility\n\n\
        The **{}** codec cannot be used with **{}** containers.\n\n\
        ### Compatible Containers\n{}\n\n\
        ### Solution\n\
        Change the output file extension to use a compatible container format.",
        codec_name,
        format_name,
        compatible_formats.iter()
            .map(|f| format!("- `{}`", f))
            .collect::<Vec<_>>()
            .join("\n")
    )
}

/// Generate markdown explanation for missing stream errors
pub fn explain_missing_stream(
    stream_type: &StreamType,
    operation: &str,
    available_streams: &[StreamType],
) -> String {
    format!(
        "## Missing {:?} Stream\n\n\
        The operation **{}** requires a {:?} stream, but none is available in the inputs.\n\n\
        ### Available Streams\n{}\n\n\
        ### Solution\n\
        - Use an input file that contains a {:?} stream, or\n\
        - Remove the {:?}-specific option from the command",
        stream_type,
        operation,
        stream_type,
        if available_streams.is_empty() {
            "None".to_string()
        } else {
            available_streams.iter()
                .map(|s| format!("- {:?}", s))
                .collect::<Vec<_>>()
                .join("\n")
        },
        stream_type,
        stream_type
    )
}

/// Build rich content for a diagnostic
pub fn build_rich_content(blocks: Vec<RichBlock>) -> Option<DiagnosticRich> {
    if blocks.is_empty() {
        None
    } else {
        Some(DiagnosticRich { blocks })
    }
}

// Helper functions

fn format_stream_types(streams: &[StreamType]) -> String {
    if streams.is_empty() {
        return String::new();
    }
    streams.iter()
        .map(|s| match s {
            StreamType::Video => "V",
            StreamType::Audio => "A",
            _ => "?",
        })
        .collect::<Vec<_>>()
        .join("+")
}

fn sanitize_label(s: &str) -> String {
    // Escape special characters for Mermaid
    s.replace("[", "&#91;")
        .replace("]", "&#93;")
        .replace("(", "&#40;")
        .replace(")", "&#41;")
}
