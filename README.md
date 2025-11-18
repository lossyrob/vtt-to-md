# vtt-to-md

A Rust command-line tool for converting VTT (WebVTT) transcript files from meeting platforms (Microsoft Teams, Zoom, Google Meet) into readable Markdown format with consolidated speaker segments.

## Features

- **Speaker Consolidation**: Merges consecutive cues from the same speaker into coherent paragraphs
- **Multi-line Voice Tag Support**: Properly handles VTT files with text spanning multiple lines within voice tags
- **Timestamp Sorting**: Automatically sorts out-of-order cues by timestamp (common in Teams transcripts)
- **Smart Unknown Speaker Filtering**: Automatically filters out cues without speaker attribution for Teams-style VTT files (those with `<v>` tags). Can be disabled with `--no-filter-unknown`
- **Flexible Timestamp Modes**: Include no timestamps, first timestamp per speaker turn, or all timestamps
- **Custom Speaker Labels**: Customize the label for cues without speaker attribution
- **Safe by Default**: Won't overwrite existing files without explicit `--force` flag
- **Cross-platform**: Runs on Windows, Linux, and macOS with no runtime dependencies

## Installation

### Windows MSI Installer

Download the MSI installer from [GitHub Releases](https://github.com/lossyrob/vtt-to-md/releases) and run it. Optionally enable .vtt file association during installation to convert files by double-clicking them.

### Build from Source

```bash
cargo install --path .
```

Or build manually:

```bash
cargo build --release
```

The binary will be in `target/release/vtt-to-md` (or `vtt-to-md.exe` on Windows).

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
- `--force`, `-f` - Overwrite existing output file
- `--no-clobber`, `-n` - Skip conversion if output file exists
- `--no-auto-increment` - Disable auto-increment of output filename (use with --force to overwrite)
- `--stdout` - Print Markdown to stdout instead of writing to file
- `--unknown-speaker LABEL` - Custom label for cues without speaker attribution (default: "Unknown")
- `--filter-unknown` - Explicitly filter out cues without speaker attribution (auto-enabled for Teams-style VTT)
- `--no-filter-unknown` - Disable automatic filtering for Teams-style VTT files
- `--include-timestamps MODE` - Timestamp inclusion mode: `none` (default), `first`, or `each`
- `--help`, `-h` - Display help text
- `--version`, `-V` - Display version

**Note:** By default, if the output file exists, a numbered suffix is added (e.g., `meeting.md`, `meeting (1).md`, `meeting (2).md`). Use `--no-auto-increment` to restore the old behavior.

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

## Output Format

The tool generates Markdown with speaker names in bold followed by their consolidated text:

```markdown
**Alice:** Hello everyone, thanks for joining. Let's start with the first agenda item.

**Bob:** Sounds good. I'll share my screen.
```

Consecutive cues from the same speaker are merged into single paragraphs for natural reading flow.

## Building

```bash
cargo build
```

For release builds:
```bash
cargo build --release
```

## Testing

```bash
cargo test
```

Run with verbose output:
```bash
cargo test -- --nocapture
```

## Development

This project uses Rust 2024 edition. Make sure you have Rust installed:

```bash
rustup update
```

## Documentation

For comprehensive documentation including architecture, design decisions, and testing guide, see [Docs.md](.paw/work/vtt-to-md-cli/Docs.md).

## License

See LICENSE file for details.
