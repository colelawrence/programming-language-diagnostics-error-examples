use crate::ast::{span_from_pest, *};
use pest::Parser;
use pest_derive::Parser;

#[derive(Parser)]
#[grammar = "ffmpeg.pest"]
pub struct FfmpegParser;

/// Parse FFmpeg command and return AST with line/column offsets for error reporting
pub fn parse_command(input: &str, line_offset: usize, column_offset: usize) -> Result<FfmpegCommand, String> {
    let pairs = FfmpegParser::parse(Rule::command, input)
        .map_err(|e| format!("Parse error: {}", e))?;
    
    let mut global_options = Vec::new();
    let mut inputs = Vec::new();
    let mut outputs = Vec::new();
    
    let command_pair = pairs.into_iter().next().unwrap();
    let command_span = span_from_pest(command_pair.as_span(), line_offset, column_offset);
    
    for pair in command_pair.into_inner() {
        match pair.as_rule() {
            Rule::global_options => {
                for option_pair in pair.into_inner() {
                    if let Some(opt) = parse_option(option_pair, line_offset, column_offset) {
                        global_options.push(opt);
                    }
                }
            }
            Rule::input_section => {
                inputs.push(parse_input_section(pair, line_offset, column_offset)?);
            }
            Rule::output_section => {
                outputs.push(parse_output_section(pair, line_offset, column_offset)?);
            }
            Rule::EOI => {}
            _ => {}
        }
    }
    
    Ok(FfmpegCommand {
        global_options,
        inputs,
        outputs,
        span: command_span,
    })
}

fn parse_input_section(pair: pest::iterators::Pair<Rule>, line_offset: usize, column_offset: usize) -> Result<InputSpec, String> {
    let span = span_from_pest(pair.as_span(), line_offset, column_offset);
    let mut options = Vec::new();
    let mut file_path = String::new();
    let mut file_path_span = span.clone();
    
    for inner in pair.into_inner() {
        match inner.as_rule() {
            Rule::input_options => {
                // input_options is a single option, get its inner 'option' rule
                for option_pair in inner.into_inner() {
                    if option_pair.as_rule() == Rule::option {
                        if let Some(opt) = parse_option(option_pair, line_offset, column_offset) {
                            options.push(opt);
                        }
                    }
                }
            }
            Rule::file_path => {
                file_path_span = span_from_pest(inner.as_span(), line_offset, column_offset);
                file_path = extract_string_value(inner);
            }
            _ => {}
        }
    }
    
    Ok(InputSpec {
        options,
        file_path,
        file_path_span,
        span,
    })
}

fn parse_output_section(pair: pest::iterators::Pair<Rule>, line_offset: usize, column_offset: usize) -> Result<OutputSpec, String> {
    let span = span_from_pest(pair.as_span(), line_offset, column_offset);
    let mut options = Vec::new();
    let mut file_path = String::new();
    let mut file_path_span = span.clone();
    
    for inner in pair.into_inner() {
        match inner.as_rule() {
            Rule::output_options => {
                // output_options is a single option, get its inner 'option' rule
                for option_pair in inner.into_inner() {
                    if option_pair.as_rule() == Rule::option {
                        if let Some(opt) = parse_option(option_pair, line_offset, column_offset) {
                            options.push(opt);
                        }
                    }
                }
            }
            Rule::file_path => {
                file_path_span = span_from_pest(inner.as_span(), line_offset, column_offset);
                file_path = extract_string_value(inner);
            }
            _ => {}
        }
    }
    
    Ok(OutputSpec {
        options,
        file_path,
        file_path_span,
        span,
    })
}

fn parse_option(pair: pest::iterators::Pair<Rule>, line_offset: usize, column_offset: usize) -> Option<OptionNode> {
    // If this is an 'option' rule, unwrap it to get the actual option type
    let actual_pair = if pair.as_rule() == Rule::option {
        pair.into_inner().next()?
    } else {
        pair
    };
    
    let span = span_from_pest(actual_pair.as_span(), line_offset, column_offset);
    
    match actual_pair.as_rule() {
        Rule::codec_option => {
            let option_text = actual_pair.as_str().to_string();
            let mut codec = String::new();
            let mut codec_span = span.clone();
            
            for inner in actual_pair.into_inner() {
                match inner.as_rule() {
                    Rule::codec_name => {
                        codec_span = span_from_pest(inner.as_span(), line_offset, column_offset);
                        codec = inner.as_str().to_string();
                    }
                    _ => {}
                }
            }
            if option_text.contains(":v") || option_text.contains("vcodec") {
                Some(OptionNode::VideoCodec { codec, codec_span, span })
            } else if option_text.contains(":a") || option_text.contains("acodec") {
                Some(OptionNode::AudioCodec { codec, codec_span, span })
            } else {
                Some(OptionNode::Codec { codec, codec_span, span })
            }
        }
        
        Rule::bitrate_option => {
            let option_text = actual_pair.as_str().to_string();
            let mut bitrate = String::new();
            let mut bitrate_span = span.clone();
            
            for inner in actual_pair.into_inner() {
                if inner.as_rule() == Rule::bitrate {
                    bitrate_span = span_from_pest(inner.as_span(), line_offset, column_offset);
                    bitrate = inner.as_str().to_string();
                }
            }
            if option_text.contains(":v") || option_text.contains("-vb") {
                Some(OptionNode::VideoBitrate { bitrate, bitrate_span, span })
            } else {
                Some(OptionNode::AudioBitrate { bitrate, bitrate_span, span })
            }
        }
        
        Rule::resolution_option => {
            let mut resolution = String::new();
            let mut resolution_span = span.clone();
            
            for inner in actual_pair.into_inner() {
                if inner.as_rule() == Rule::resolution {
                    resolution_span = span_from_pest(inner.as_span(), line_offset, column_offset);
                    resolution = inner.as_str().to_string();
                }
            }
            
            Some(OptionNode::Resolution { resolution, resolution_span, span })
        }
        
        Rule::framerate_option => {
            let mut rate = String::new();
            let mut rate_span = span.clone();
            
            for inner in actual_pair.into_inner() {
                if inner.as_rule() == Rule::number {
                    rate_span = span_from_pest(inner.as_span(), line_offset, column_offset);
                    rate = inner.as_str().to_string();
                }
            }
            
            Some(OptionNode::FrameRate { rate, rate_span, span })
        }
        
        Rule::video_filter_option => {
            let filter = parse_filter_spec(actual_pair.clone(), line_offset, column_offset);
            Some(OptionNode::VideoFilter { filter, span })
        }
        
        Rule::audio_filter_option => {
            let filter = parse_filter_spec(actual_pair.clone(), line_offset, column_offset);
            Some(OptionNode::AudioFilter { filter, span })
        }
        
        Rule::filter_complex_option => {
            let filter = parse_filter_spec(actual_pair.clone(), line_offset, column_offset);
            Some(OptionNode::FilterComplex { filter, span })
        }
        
        Rule::map_option => {
            let mut mapping = String::new();
            let mut mapping_span = span.clone();
            
            for inner in actual_pair.into_inner() {
                if inner.as_rule() == Rule::map_specifier {
                    mapping_span = span_from_pest(inner.as_span(), line_offset, column_offset);
                    mapping = inner.as_str().to_string();
                }
            }
            
            Some(OptionNode::Map { mapping, mapping_span, span })
        }
        
        Rule::format_option => {
            let mut format = String::new();
            let mut format_span = span.clone();
            
            for inner in actual_pair.into_inner() {
                if inner.as_rule() == Rule::format_name {
                    format_span = span_from_pest(inner.as_span(), line_offset, column_offset);
                    format = inner.as_str().to_string();
                }
            }
            
            Some(OptionNode::Format { format, format_span, span })
        }
        
        Rule::time_option => {
            let option_text = actual_pair.as_str().to_string();
            let mut time = String::new();
            let mut time_span = span.clone();
            
            for inner in actual_pair.into_inner() {
                if inner.as_rule() == Rule::time_value {
                    time_span = span_from_pest(inner.as_span(), line_offset, column_offset);
                    time = inner.as_str().to_string();
                }
            }
            
            if option_text.starts_with("-ss") {
                Some(OptionNode::SeekStart { time, time_span, span })
            } else {
                Some(OptionNode::Duration { time, time_span, span })
            }
        }
        
        Rule::stream_option => {
            let option_text = actual_pair.as_str().to_string();
            let mut value = String::new();
            let mut value_span = span.clone();
            
            for inner in actual_pair.into_inner() {
                if inner.as_rule() == Rule::number {
                    value_span = span_from_pest(inner.as_span(), line_offset, column_offset);
                    value = inner.as_str().to_string();
                }
            }
            
            if option_text.starts_with("-ar") {
                Some(OptionNode::SampleRate { rate: value, rate_span: value_span, span })
            } else {
                Some(OptionNode::AudioChannels { channels: value, channels_span: value_span, span })
            }
        }
        
        Rule::flag => {
            let pair_str = actual_pair.as_str().to_string();
            let mut name = String::new();
            let mut value = None;
            let mut value_span = None;
            
            for inner in actual_pair.into_inner() {
                match inner.as_rule() {
                    Rule::flag_name => {
                        name = format!("-{}", inner.as_str());
                    }
                    Rule::flag_value => {
                        value_span = Some(span_from_pest(inner.as_span(), line_offset, column_offset));
                        value = Some(extract_string_value(inner));
                    }
                    _ => {}
                }
            }
            
            if name.is_empty() {
                name = pair_str;
            }
            
            Some(OptionNode::Generic { name, value, value_span, span })
        }
        
        _ => None,
    }
}

fn parse_filter_spec(pair: pest::iterators::Pair<Rule>, line_offset: usize, column_offset: usize) -> FilterSpec {
    let span = span_from_pest(pair.as_span(), line_offset, column_offset);
    let mut raw = String::new();
    
    for inner in pair.into_inner() {
        match inner.as_rule() {
            Rule::filter_graph | Rule::unquoted_filter => {
                raw = extract_string_value(inner);
            }
            _ => {}
        }
    }
    
    FilterSpec {
        raw: raw.clone(),
        parsed: None, // TODO: Implement filter graph parsing
        span,
    }
}

fn extract_string_value(pair: pest::iterators::Pair<Rule>) -> String {
    let text = pair.as_str();
    
    // Remove quotes if present
    if (text.starts_with('"') && text.ends_with('"')) 
        || (text.starts_with('\'') && text.ends_with('\'')) {
        text[1..text.len()-1].to_string()
    } else {
        text.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_parse_simple_command() {
        let input = "ffmpeg -i input.mp4 output.mp4";
        let result = parse_command(input, 0, 0);
        assert!(result.is_ok());
        let cmd = result.unwrap();
        assert_eq!(cmd.inputs.len(), 1);
        assert_eq!(cmd.outputs.len(), 1);
        assert_eq!(cmd.inputs[0].file_path, "input.mp4");
        assert_eq!(cmd.outputs[0].file_path, "output.mp4");
    }
    
    #[test]
    fn test_parse_with_codec() {
        let input = "ffmpeg -i input.mp4 -c:v libx264 output.mp4";
        let result = parse_command(input, 0, 0);
        assert!(result.is_ok());
        let cmd = result.unwrap();
        assert_eq!(cmd.outputs[0].options.len(), 1);
    }
    
    #[test]
    fn test_parse_with_filter() {
        let input = "ffmpeg -i input.mp4 -vf scale=1920:1080 output.mp4";
        let result = parse_command(input, 0, 0);
        assert!(result.is_ok());
    }
}

