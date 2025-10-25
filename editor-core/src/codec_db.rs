use shared_types::StreamType;
use std::collections::HashMap;

/// Codec information
#[derive(Debug, Clone)]
pub struct CodecInfo {
    pub name: String,
    pub stream_type: StreamType,
    pub is_encoder: bool,
    pub is_decoder: bool,
}

/// Format (container) information
#[derive(Debug, Clone)]
pub struct FormatInfo {
    pub name: String,
    pub supported_video_codecs: Vec<String>,
    pub supported_audio_codecs: Vec<String>,
    pub extensions: Vec<String>,
}

/// Filter information
#[derive(Debug, Clone)]
pub struct FilterInfo {
    pub name: String,
    pub input_type: StreamType,
    pub output_type: StreamType,
    pub description: String,
}

/// Static codec database
pub struct CodecDatabase {
    codecs: HashMap<String, CodecInfo>,
    formats: HashMap<String, FormatInfo>,
    filters: HashMap<String, FilterInfo>,
}

impl CodecDatabase {
    pub fn new() -> Self {
        let mut db = CodecDatabase {
            codecs: HashMap::new(),
            formats: HashMap::new(),
            filters: HashMap::new(),
        };
        
        db.init_codecs();
        db.init_formats();
        db.init_filters();
        
        db
    }
    
    fn init_codecs(&mut self) {
        // Video codecs
        let video_codecs = vec![
            "libx264", "libx265", "h264", "hevc", "vp8", "vp9", "av1", "libaom-av1",
            "mpeg4", "mpeg2video", "libvpx", "libvpx-vp9", "prores", "dnxhd",
            "mjpeg", "png", "rawvideo", "copy",
        ];
        
        for codec in video_codecs {
            self.codecs.insert(codec.to_string(), CodecInfo {
                name: codec.to_string(),
                stream_type: StreamType::Video,
                is_encoder: true,
                is_decoder: true,
            });
        }
        
        // Audio codecs
        let audio_codecs = vec![
            "aac", "libfdk_aac", "mp3", "libmp3lame", "opus", "libopus",
            "vorbis", "libvorbis", "flac", "alac", "ac3", "eac3",
            "pcm_s16le", "pcm_s24le", "pcm_f32le", "copy",
        ];
        
        for codec in audio_codecs {
            self.codecs.insert(codec.to_string(), CodecInfo {
                name: codec.to_string(),
                stream_type: StreamType::Audio,
                is_encoder: true,
                is_decoder: true,
            });
        }
    }
    
    fn init_formats(&mut self) {
        // MP4 container
        self.formats.insert("mp4".to_string(), FormatInfo {
            name: "mp4".to_string(),
            supported_video_codecs: vec![
                "h264".to_string(), "hevc".to_string(), "mpeg4".to_string(),
                "libx264".to_string(), "libx265".to_string(),
            ],
            supported_audio_codecs: vec![
                "aac".to_string(), "mp3".to_string(), "ac3".to_string(),
            ],
            extensions: vec!["mp4".to_string(), "m4v".to_string()],
        });
        
        // WebM container
        self.formats.insert("webm".to_string(), FormatInfo {
            name: "webm".to_string(),
            supported_video_codecs: vec![
                "vp8".to_string(), "vp9".to_string(), "av1".to_string(),
                "libvpx".to_string(), "libvpx-vp9".to_string(),
            ],
            supported_audio_codecs: vec![
                "opus".to_string(), "vorbis".to_string(), "libopus".to_string(),
            ],
            extensions: vec!["webm".to_string()],
        });
        
        // MKV container (Matroska) - very permissive
        self.formats.insert("matroska".to_string(), FormatInfo {
            name: "matroska".to_string(),
            supported_video_codecs: vec![
                "h264".to_string(), "hevc".to_string(), "vp8".to_string(), 
                "vp9".to_string(), "av1".to_string(), "mpeg4".to_string(),
            ],
            supported_audio_codecs: vec![
                "aac".to_string(), "mp3".to_string(), "opus".to_string(),
                "vorbis".to_string(), "flac".to_string(), "ac3".to_string(),
            ],
            extensions: vec!["mkv".to_string(), "mka".to_string()],
        });
        
        // AVI container
        self.formats.insert("avi".to_string(), FormatInfo {
            name: "avi".to_string(),
            supported_video_codecs: vec![
                "mpeg4".to_string(), "h264".to_string(), "mjpeg".to_string(),
            ],
            supported_audio_codecs: vec![
                "mp3".to_string(), "ac3".to_string(), "pcm_s16le".to_string(),
            ],
            extensions: vec!["avi".to_string()],
        });
        
        // MOV container (QuickTime)
        self.formats.insert("mov".to_string(), FormatInfo {
            name: "mov".to_string(),
            supported_video_codecs: vec![
                "h264".to_string(), "hevc".to_string(), "prores".to_string(),
                "mpeg4".to_string(),
            ],
            supported_audio_codecs: vec![
                "aac".to_string(), "alac".to_string(), "pcm_s16le".to_string(),
            ],
            extensions: vec!["mov".to_string(), "qt".to_string()],
        });
    }
    
    fn init_filters(&mut self) {
        // Video filters
        let video_filters = vec![
            ("scale", "Resize video", StreamType::Video, StreamType::Video),
            ("crop", "Crop video", StreamType::Video, StreamType::Video),
            ("pad", "Add padding to video", StreamType::Video, StreamType::Video),
            ("rotate", "Rotate video", StreamType::Video, StreamType::Video),
            ("hflip", "Flip video horizontally", StreamType::Video, StreamType::Video),
            ("vflip", "Flip video vertically", StreamType::Video, StreamType::Video),
            ("fps", "Change frame rate", StreamType::Video, StreamType::Video),
            ("format", "Convert pixel format", StreamType::Video, StreamType::Video),
            ("overlay", "Overlay one video on another", StreamType::Video, StreamType::Video),
            ("drawtext", "Draw text on video", StreamType::Video, StreamType::Video),
            ("colorbalance", "Adjust color balance", StreamType::Video, StreamType::Video),
            ("eq", "Adjust brightness/contrast", StreamType::Video, StreamType::Video),
        ];
        
        for (name, desc, in_type, out_type) in video_filters {
            self.filters.insert(name.to_string(), FilterInfo {
                name: name.to_string(),
                input_type: in_type,
                output_type: out_type,
                description: desc.to_string(),
            });
        }
        
        // Audio filters
        let audio_filters = vec![
            ("volume", "Adjust audio volume", StreamType::Audio, StreamType::Audio),
            ("atempo", "Adjust audio tempo", StreamType::Audio, StreamType::Audio),
            ("aresample", "Resample audio", StreamType::Audio, StreamType::Audio),
            ("aformat", "Convert audio format", StreamType::Audio, StreamType::Audio),
            ("loudnorm", "Normalize audio loudness", StreamType::Audio, StreamType::Audio),
            ("equalizer", "Audio equalizer", StreamType::Audio, StreamType::Audio),
            ("highpass", "High-pass filter", StreamType::Audio, StreamType::Audio),
            ("lowpass", "Low-pass filter", StreamType::Audio, StreamType::Audio),
            ("pan", "Audio channel mapping", StreamType::Audio, StreamType::Audio),
        ];
        
        for (name, desc, in_type, out_type) in audio_filters {
            self.filters.insert(name.to_string(), FilterInfo {
                name: name.to_string(),
                input_type: in_type,
                output_type: out_type,
                description: desc.to_string(),
            });
        }
    }
    
    pub fn get_codec(&self, name: &str) -> Option<&CodecInfo> {
        self.codecs.get(name)
    }
    
    pub fn get_format(&self, name: &str) -> Option<&FormatInfo> {
        self.formats.get(name)
    }
    
    pub fn get_format_by_extension(&self, ext: &str) -> Option<&FormatInfo> {
        self.formats.values().find(|f| f.extensions.contains(&ext.to_string()))
    }
    
    pub fn get_filter(&self, name: &str) -> Option<&FilterInfo> {
        self.filters.get(name)
    }
    
    pub fn is_codec_supported_in_format(&self, codec: &str, format: &str) -> bool {
        if let Some(codec_info) = self.get_codec(codec) {
            if let Some(format_info) = self.get_format(format) {
                match codec_info.stream_type {
                    StreamType::Video => {
                        return format_info.supported_video_codecs.contains(&codec.to_string());
                    }
                    StreamType::Audio => {
                        return format_info.supported_audio_codecs.contains(&codec.to_string());
                    }
                    _ => return false,
                }
            }
        }
        false
    }
    
    pub fn infer_format_from_filename(&self, filename: &str) -> Option<String> {
        if let Some(ext) = filename.split('.').last() {
            if let Some(format_info) = self.get_format_by_extension(ext) {
                return Some(format_info.name.clone());
            }
        }
        None
    }
}

impl Default for CodecDatabase {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_codec_lookup() {
        let db = CodecDatabase::new();
        let codec = db.get_codec("libx264");
        assert!(codec.is_some());
        assert!(matches!(codec.unwrap().stream_type, StreamType::Video));
    }
    
    #[test]
    fn test_format_compatibility() {
        let db = CodecDatabase::new();
        assert!(db.is_codec_supported_in_format("libx264", "mp4"));
        assert!(!db.is_codec_supported_in_format("vp9", "mp4"));
    }
    
    #[test]
    fn test_infer_format() {
        let db = CodecDatabase::new();
        assert_eq!(db.infer_format_from_filename("video.mp4"), Some("mp4".to_string()));
        assert_eq!(db.infer_format_from_filename("video.webm"), Some("webm".to_string()));
    }
}

