# vtt-to-md

A Rust command-line tool for converting VTT (WebVTT) transcript files from meeting platforms (Microsoft Teams, Zoom, Google Meet) into readable Markdown format with consolidated speaker segments.

## Features

- **Speaker Consolidation**: Merges consecutive cues from the same speaker into coherent paragraphs
- **Multi-line Voice Tag Support**: Properly handles VTT files with text spanning multiple lines within voice tags
- **Timestamp Sorting**: Automatically sorts out-of-order cues by timestamp (common in Teams transcripts)
- **Smart Unknown Speaker Filtering**: Automatically filters out cues without speaker attribution for Teams-style VTT files (those with `<v>` tags). Can be disabled with `--no-filter-unknown`
- **Flexible Timestamp Modes**: Include no timestamps, first timestamp per speaker turn, or all timestamps
- **Custom Speaker Labels**: Customize the label for cues without speaker attribution

## Installation

```bash
cargo install --path .
```

Or build from source:

```bash
cargo build --release
```

## Usage

Basic conversion:
```bash
vtt-to-md input.vtt
```

With options:
```bash
vtt-to-md input.vtt output.md --filter-unknown --include-timestamps first
```

### Command-Line Options

- `INPUT` - Path to the input VTT file (required)
- `OUTPUT` - Path to the output Markdown file (optional, defaults to INPUT with .md extension)
- `--force` - Overwrite existing output file
- `--no-clobber` - Skip conversion if output file exists
- `--stdout` - Print Markdown to stdout instead of writing to file
- `--unknown-speaker LABEL` - Custom label for cues without speaker attribution (default: "Unknown")
- `--filter-unknown` - Explicitly filter out cues without speaker attribution (auto-enabled for Teams-style VTT)
- `--no-filter-unknown` - Disable automatic filtering for Teams-style VTT files
- `--include-timestamps MODE` - Timestamp inclusion mode: `none` (default), `first`, or `each`

### Examples

Convert Teams transcript (Unknown speakers automatically filtered):
```bash
vtt-to-md "teams-meeting.vtt"
```

Keep Unknown speakers for Teams transcript:
```bash
vtt-to-md "teams-meeting.vtt" --no-filter-unknown
```

Include first timestamp per speaker turn:
```bash
vtt-to-md "meeting.vtt" --include-timestamps first
```

Output to stdout with custom unknown speaker label:
```bash
vtt-to-md "meeting.vtt" --stdout --unknown-speaker "Narrator"
```

Force overwrite existing file:
```bash
vtt-to-md "meeting.vtt" "notes.md" --force
```

## Building

```bash
cargo build
```

## Testing

```bash
cargo test
```

## Development

This project uses Rust 2024 edition. Make sure you have Rust installed:

```bash
rustup update
```
