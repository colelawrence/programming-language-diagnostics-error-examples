# FFmpeg Language Specification

## Overview

This editor provides real-time analysis and diagnostics for FFmpeg commands. It validates command syntax, checks codec/format compatibility, tracks stream types through the processing pipeline, and detects common errors before you run the command.

## Supported FFmpeg Syntax

The editor supports the core FFmpeg command structure covering approximately 80% of common use cases:

```bash
ffmpeg [global_options] {[input_options] -i input_file} {[output_options] output_file}
```

### Global Options

Options that apply to the entire FFmpeg command:
- `-y` - Overwrite output files without asking
- `-n` - Never overwrite output files
- `-v` - Set logging level
- `-hide_banner` - Hide startup banner
- `-stats` - Show encoding statistics

### Input Options

Options that apply to input files (placed before `-i`):
- `-f FORMAT` - Force input format
- `-ss TIME` - Seek to position before reading input
- `-t DURATION` - Limit duration of data read from input
- `-stream_loop N` - Loop input stream N times

### Output Options

Options that apply to output files:

**Video Options:**
- `-c:v CODEC` or `-vcodec CODEC` - Video codec (e.g., libx264, libx265, vp9)
- `-b:v BITRATE` or `-vb BITRATE` - Video bitrate (e.g., 2M, 5000k)
- `-s WxH` - Resolution (e.g., 1920x1080)
- `-r FPS` - Frame rate (e.g., 30, 60)
- `-vf FILTERS` - Video filters (e.g., scale=1920:1080)

**Audio Options:**
- `-c:a CODEC` or `-acodec CODEC` - Audio codec (e.g., aac, mp3, opus)
- `-b:a BITRATE` or `-ab BITRATE` - Audio bitrate (e.g., 128k, 320k)
- `-ar RATE` - Audio sample rate (e.g., 44100, 48000)
- `-ac CHANNELS` - Audio channels (e.g., 2 for stereo)
- `-af FILTERS` - Audio filters (e.g., volume=2.0)

**Stream Mapping:**
- `-map SPECIFIER` - Select streams for output (e.g., 0:v:0, 0:a)

**Format:**
- `-f FORMAT` - Force output format

**Complex Filters:**
- `-filter_complex GRAPH` - Complex filter graph for multiple inputs/outputs

## Stream Type Tracking

The editor tracks stream types (video, audio, subtitle) through the FFmpeg pipeline:

1. **Input Analysis**: Determines available streams from input files based on file extensions
2. **Filter Validation**: Verifies filters are applied to compatible stream types
3. **Codec Validation**: Ensures codecs match their target stream types
4. **Output Validation**: Checks that output operations reference available streams

### Example Stream Type Flow

```bash
# Input has video + audio streams
ffmpeg -i video.mp4 \
  -vf scale=1920:1080 \    # Video filter (requires video stream)
  -c:v libx264 \           # Video codec (operates on video stream)
  -af volume=2.0 \         # Audio filter (requires audio stream)
  -c:a aac \               # Audio codec (operates on audio stream)
  output.mp4
```

## Error Codes and Categories

### E100-E199: Stream Type Mismatches

| Code | Description | Example |
|------|-------------|---------|
| E101 | Video filter applied to audio-only stream | `ffmpeg -i audio.mp3 -vf scale=640:480 output.mp4` |
| E102 | Audio filter applied to video-only stream | `ffmpeg -i image.png -af volume=2.0 output.png` |
| E104 | No video stream available for video operations | `ffmpeg -i audio.mp3 -c:v libx264 output.mp4` |
| E105 | No audio stream available for audio operations | `ffmpeg -i video.mp4 -c:a aac output.mp4` (when video has no audio) |

### E200-E299: Codec/Format Incompatibilities

| Code | Description | Example |
|------|-------------|---------|
| E201 | Codec not supported in output container | `ffmpeg -i input.mp4 -c:v vp9 output.mp4` (VP9 not in MP4) |
| E205 | Invalid codec for stream type | Using video codec for audio stream |

### E300-E399: Stream Mapping Errors

| Code | Description | Example |
|------|-------------|---------|
| E301 | Mapped stream does not exist | `ffmpeg -i input.mp4 -map 2:0 output.mp4` (only 1 input) |
| E303 | Filter output label does not exist | Referencing non-existent filter label |

### E400-E499: Parameter/Option Errors

| Code | Description | Example |
|------|-------------|---------|
| E401 | Invalid resolution format | `ffmpeg -i input.mp4 -s 1920 output.mp4` (missing height) |
| E402 | Invalid bitrate format | `ffmpeg -i input.mp4 -b:v abc output.mp4` (not a number) |
| E403 | Invalid frame rate value | `ffmpeg -i input.mp4 -r -1 output.mp4` (negative fps) |

### E500-E599: Filter Syntax Errors

| Code | Description | Example |
|------|-------------|---------|
| E502 | Unknown filter name | `ffmpeg -i input.mp4 -vf nonexistent output.mp4` |

### W100-W199: Performance/Quality Warnings

| Code | Description | Example |
|------|-------------|---------|
| W101 | Extremely high bitrate specified | `ffmpeg -i input.mp4 -b:v 100M output.mp4` |
| W201 | Unknown codec | Using a codec not in the database |

## Supported Codecs and Formats

### Video Codecs
- H.264: `libx264`, `h264`
- H.265/HEVC: `libx265`, `hevc`
- VP8: `vp8`, `libvpx`
- VP9: `vp9`, `libvpx-vp9`
- AV1: `av1`, `libaom-av1`
- MPEG-4: `mpeg4`
- ProRes: `prores`

### Audio Codecs
- AAC: `aac`, `libfdk_aac`
- MP3: `mp3`, `libmp3lame`
- Opus: `opus`, `libopus`
- Vorbis: `vorbis`, `libvorbis`
- FLAC: `flac`
- AC3: `ac3`, `eac3`

### Container Formats
- MP4: `.mp4` - Supports H.264, H.265, AAC, MP3
- WebM: `.webm` - Supports VP8, VP9, AV1, Opus, Vorbis
- Matroska: `.mkv` - Very permissive, supports most codecs
- AVI: `.avi` - Supports MPEG-4, H.264, MP3, AC3
- MOV: `.mov` - Supports H.264, H.265, ProRes, AAC, ALAC

## Common Filters

### Video Filters
- `scale=W:H` - Resize video to width x height
- `crop=W:H:X:Y` - Crop video
- `rotate=ANGLE` - Rotate video
- `hflip` - Flip horizontally
- `vflip` - Flip vertically
- `fps=RATE` - Change frame rate
- `eq=brightness=VALUE` - Adjust brightness/contrast

### Audio Filters
- `volume=VALUE` - Adjust volume
- `atempo=RATE` - Change tempo
- `loudnorm` - Normalize audio loudness
- `highpass=f=FREQ` - High-pass filter
- `lowpass=f=FREQ` - Low-pass filter

## Examples

### Valid Commands

**Simple Transcode:**
```bash
ffmpeg -i input.mp4 output.mp4
```

**Convert with Specific Codec:**
```bash
ffmpeg -i input.mp4 -c:v libx264 -c:a aac output.mp4
```

**Resize Video:**
```bash
ffmpeg -i input.mp4 -vf scale=1280:720 -c:v libx264 output.mp4
```

**Extract Audio:**
```bash
ffmpeg -i video.mp4 -vn -c:a copy audio.aac
```

### Commands with Errors

**Video Codec on Audio File:**
```bash
# ERROR E104: No video stream available
ffmpeg -i audio.mp3 -c:v libx264 output.mp4
```

**Incompatible Codec/Format:**
```bash
# ERROR E201: VP9 not supported in MP4
ffmpeg -i input.mp4 -c:v vp9 output.mp4
```

**Invalid Resolution:**
```bash
# ERROR E401: Invalid resolution format
ffmpeg -i input.mp4 -s 1920 output.mp4
```

**Video Filter on Audio:**
```bash
# ERROR E101: Video filter on audio-only stream
ffmpeg -i audio.mp3 -vf scale=640:480 output.mp4
```

## Implementation Notes

### Parser
The editor uses a PEG (Parsing Expression Grammar) parser built with Rust's `pest` library. The grammar is defined in `editor-core/src/ffmpeg.pest` and covers the core FFmpeg command syntax.

### Analyzer
The semantic analyzer performs multiple passes:
1. Stream discovery from input files
2. Stream type tracking through filters
3. Codec/format compatibility validation
4. Parameter validation

### Limitations
- File analysis is based on extension, not actual content
- Complex filter graphs are validated at a basic level
- Some advanced FFmpeg features are not yet supported
- Codec/format database is a curated subset of FFmpeg's full capabilities

