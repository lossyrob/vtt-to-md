# VTT to MD CLI - Documentation

## Overview

VTT to MD CLI is a Rust-based command-line tool that converts WebVTT (Web Video Text Tracks) transcript files into readable Markdown documents. Meeting platforms like Microsoft Teams, Zoom, and Google Meet export transcripts as VTT files—a technical format filled with timestamps and formatting tags that's difficult to read, search, or share. This tool bridges that gap by parsing the VTT structure, extracting speaker segments, consolidating consecutive utterances from the same speaker, and generating clean, well-formatted Markdown suitable for documentation, archiving, or further editing.

The tool solves a real workflow pain point: transforming machine-generated transcripts into human-readable documents. Users can invoke it from the terminal (`vtt-to-md meeting.vtt`) or configure their operating system to automatically convert VTT files on double-click. The resulting Markdown uses bold speaker names followed by consolidated text, creating a natural reading experience that mirrors traditional meeting notes.

## Architecture and Design

### High-Level Architecture

The tool follows a classic pipeline architecture with clear separation of concerns across five modules:

```
VTT File → Parser → Cues → Consolidator → Segments → Markdown → Output (File/Stdout)
              ↓                                          ↓
         Validation                               Error Handling
```

Each stage transforms data and passes it to the next, with comprehensive error handling at every step. This modular design enables independent testing of each component and makes the codebase maintainable and extensible.

### Design Decisions

**Error Handling Strategy**

The tool uses a dual-library approach for error handling:
- **thiserror** defines the `VttError` enum with structured error variants for library-level errors
- **anyhow** provides error context chaining at the application level

Exit codes follow BSD `sysexits.h` conventions for cross-platform consistency and script-friendliness:
- 0: Success
- 64 (EX_USAGE): CLI usage errors (invalid arguments, conflicting flags)
- 65 (EX_DATAERR): VTT parse errors (invalid format, missing WEBVTT header)
- 66 (EX_NOINPUT): Input file not found
- 73 (EX_CANTCREAT): Output file already exists (without --force)
- 74 (EX_IOERR): General I/O errors
- 77 (EX_NOPERM): Permission denied

This explicit mapping ensures the tool integrates well with shell scripts and automation workflows.

**Parsing Strategy**

The parser uses regex patterns to extract speaker information from VTT voice tags (`<v Speaker>text</v>`). A critical design decision was to be **permissive** rather than strict:
- Handles both closed `<v Speaker>text</v>` and unclosed `<v Speaker>text` patterns
- Supports multi-line text within voice tags using `(?s)` DOTALL regex flag
- Sorts cues by timestamp to handle out-of-order entries (common in Teams exports)
- Skips unknown VTT blocks (NOTE, STYLE, REGION) without failing
- Decodes HTML entities (&amp;, &lt;, &gt;, etc.) to literal characters

This permissive approach ensures the tool works reliably with real-world transcripts from different platforms without requiring manual preprocessing.

**Smart Unknown Speaker Filtering**

One of the most innovative features is **automatic unknown speaker filtering** for Teams-style VTT files. The tool detects when a VTT file contains voice tags (indicating it follows the Teams format) and automatically filters out cues without speaker attribution. This eliminates noise from the output without requiring users to specify flags.

The detection logic:
1. Parse the VTT file and identify if any cues have speaker attribution
2. If voice tags are present (Teams format): auto-enable filtering
3. User can override with `--no-filter-unknown` if they want to keep unknown speakers
4. User can explicitly enable filtering for any file with `--filter-unknown`

This behavior significantly improves usability for the most common use case (Teams transcripts) while maintaining flexibility for other scenarios.

**Speaker Consolidation**

The consolidation algorithm groups consecutive cues from the same speaker into coherent paragraphs. Key design choices:
- Detects speaker changes by comparing current speaker with previous
- Joins text with single spaces between cues
- Preserves sentence boundaries (respects terminal punctuation)
- Trims final consolidated text to remove leading/trailing whitespace

This produces natural-sounding paragraphs that mirror how humans would format a transcript manually.

**Output Format and Safety**

The tool defaults to **safe behavior** by refusing to overwrite existing files unless explicitly instructed with `--force`. This prevents accidental data loss. The default output location is the same directory as the input file, with the `.md` extension, making it easy to find the converted file.

### Integration Points

The tool is designed as a standalone executable with no runtime dependencies beyond the OS standard library. This simplicity enables:
- **File association workflows**: Users can configure their OS to associate `.vtt` files with the tool
- **Script integration**: Clear exit codes and stdout mode support piping to other tools
- **Cross-platform compatibility**: Compiles to native binaries for Windows, Linux, and macOS

## User Guide

### Prerequisites

- Rust toolchain (for building from source)
- VTT transcript files from supported platforms (Teams, Zoom, Google Meet)

### Basic Usage

Convert a VTT file to Markdown:
```bash
vtt-to-md meeting.vtt
```

This creates `meeting.md` in the same directory with consolidated speaker segments.

Specify custom output path:
```bash
vtt-to-md meeting.vtt notes/transcript.md
```

### Advanced Usage

**Filter unknown speakers** (automatically enabled for Teams transcripts):
```bash
vtt-to-md teams-meeting.vtt
# Unknown speakers filtered automatically
```

**Keep unknown speakers** in Teams transcript:
```bash
vtt-to-md teams-meeting.vtt --no-filter-unknown
```

**Include timestamps** (first timestamp per speaker turn):
```bash
vtt-to-md meeting.vtt --include-timestamps first
```

**Output to stdout** for piping:
```bash
vtt-to-md meeting.vtt --stdout | grep "important topic"
```

**Force overwrite** existing file:
```bash
vtt-to-md meeting.vtt --force
```

**Custom unknown speaker label**:
```bash
vtt-to-md meeting.vtt --unknown-speaker "Unidentified"
```

### Configuration

The tool accepts the following command-line options:

**Positional Arguments:**
- `INPUT` (required): Path to the input VTT file
- `OUTPUT` (optional): Path to the output Markdown file (defaults to INPUT with .md extension)

**Flags:**
- `--force`, `-f`: Overwrite existing output file
- `--no-clobber`, `-n`: Skip conversion if output file exists
- `--stdout`: Print Markdown to stdout instead of writing to file
- `--filter-unknown`: Explicitly filter out cues without speaker attribution
- `--no-filter-unknown`: Disable automatic filtering for Teams-style VTT files

**Options:**
- `--unknown-speaker <LABEL>`: Custom label for cues without speaker attribution (default: "Unknown")
- `--include-timestamps <MODE>`: Timestamp inclusion mode—`none` (default), `first`, or `each`

**Built-in:**
- `--help`, `-h`: Display help text
- `--version`, `-V`: Display version

## Technical Reference

### Module Structure

**main.rs**
- Entry point and orchestration
- Parses CLI arguments with clap
- Runs conversion pipeline: parse → consolidate → format → write
- Maps errors to appropriate exit codes

**cli.rs**
- Defines `Args` struct using clap derive macros
- Validates argument combinations (detects conflicts like --force with --no-clobber)
- Derives output path from input path when not specified
- Prevents overwriting input file

**parser.rs**
- Defines `Cue` and `VttDocument` data structures
- Reads VTT files and validates WEBVTT header
- Extracts speaker names from voice tags using regex
- Cleans and sanitizes speaker names and cue text
- Sorts cues by timestamp to handle out-of-order entries
- Detects Teams format by checking for voice tags

**consolidator.rs**
- Defines `SpeakerSegment` data structure
- Consolidates consecutive same-speaker cues into paragraphs
- Joins text with intelligent whitespace handling
- Supports three timestamp modes (none, first, each)

**markdown.rs**
- Formats speaker segments as Markdown: `**SpeakerName:** text`
- Writes output to file with safety checks (existence, permissions)
- Supports stdout mode for piping

**error.rs**
- Defines `VttError` enum with structured error variants
- Maps errors to BSD sysexits.h exit codes
- Provides clear, actionable error messages with file paths

### Key Components

**Cue Parsing**

The parser identifies VTT cues by looking for timestamp lines (format: `HH:MM:SS.mmm --> HH:MM:SS.mmm`), then extracts the text lines that follow. Speaker names are extracted from voice tags using these regex patterns:

1. Closed tags with speaker: `(?s)<v\s+([^>]*)>(.*?)</v>`
2. Closed tags without speaker: `(?s)<v>(.*?)</v>`
3. Unclosed tags: `(?s)<v\s+([^>]*)>(.*)`

The `(?s)` flag enables DOTALL mode, allowing `.` to match newlines. This is critical for handling multi-line text within voice tags, which is common in Teams transcripts.

**Speaker Name Sanitization**

Speaker names undergo several transformations:
1. Remove `@` symbols (Teams uses these for anonymized users)
2. Apply Unicode NFC normalization to prevent combining character issues
3. Escape Markdown special characters (`*`, `_`, `#`, `[`, `]`, `(`, `)`, `{`, `}`, `!`, `>`, `|`, `` ` ``, `\`) with backslashes
4. Trim whitespace
5. Treat whitespace-only names as unknown speakers

**Teams Format Detection**

The parser sets a `has_voice_tags` boolean field on `VttDocument` by checking if any parsed cues have speaker attribution. This simple heuristic reliably distinguishes Teams transcripts (which consistently use `<v>` tags) from other formats.

### Error Handling

The tool never panics on expected errors. All error conditions are represented in the `VttError` enum and return appropriate exit codes:

- **FileNotFound**: Input file doesn't exist (exit 66)
- **PermissionDenied**: Read/write permission denied (exit 77)
- **ParseError**: Invalid VTT format or missing WEBVTT header (exit 65)
- **OutputExists**: Output file already exists and --force not set (exit 73)
- **SameFile**: Output path is identical to input path (exit 64)
- **IoError**: General I/O errors (exit 74)
- **UsageError**: Invalid CLI arguments or conflicting flags (exit 64)
- **WriteError**: Error writing to file or stdout (exit 74)

Each error includes relevant context (file paths, reasons) and provides actionable guidance for users.

## Usage Examples

### Example 1: Convert Teams Transcript

```bash
vtt-to-md "Weekly Team Meeting 2025-11-18.vtt"
```

**Input (VTT):**
```
WEBVTT

00:00:01.000 --> 00:00:03.000
<v Alice>Hello everyone, thanks for joining.

00:00:03.500 --> 00:00:06.000
<v Alice>Let's start with the first agenda item.

00:00:06.500 --> 00:00:09.000
<v Bob>Sounds good. I'll share my screen.
```

**Output (Markdown):**
```markdown
**Alice:** Hello everyone, thanks for joining. Let's start with the first agenda item.

**Bob:** Sounds good. I'll share my screen.
```

Note that Alice's consecutive cues are consolidated into a single paragraph.

### Example 2: Include Timestamps

```bash
vtt-to-md meeting.vtt --include-timestamps first
```

**Output (Markdown):**
```markdown
[00:00:01.000] **Alice:** Hello everyone, thanks for joining. Let's start with the first agenda item.

[00:00:06.500] **Bob:** Sounds good. I'll share my screen.
```

The timestamp shows when each speaker turn began.

### Example 3: Filter Unknown Speakers

For a Teams transcript with some cues missing speaker attribution:

```bash
vtt-to-md teams-meeting.vtt
# Automatically filters unknown speakers
```

To keep unknown speakers:

```bash
vtt-to-md teams-meeting.vtt --no-filter-unknown
```

### Example 4: Pipe to Other Tools

```bash
vtt-to-md meeting.vtt --stdout | grep -i "action item"
```

This searches the converted Markdown for "action item" without creating a file.

## Edge Cases and Limitations

### Known Limitations

**Mid-cue Speaker Changes**

The VTT format doesn't support multiple speakers within a single cue in standard usage. If a VTT file somehow contains mid-cue speaker changes, the tool will only recognize the first speaker for that cue.

**Timestamp Mode "Each" Behavior**

When using `--include-timestamps each`, the tool currently shows the first timestamp with the full consolidated text for each speaker turn. A more sophisticated approach might split the text by timestamp, but this would complicate the consolidation logic and is not necessary for the primary use case.

**File Association Configuration**

The tool doesn't automatically register file associations. Users must configure this manually per operating system:
- **Windows**: Use "Open with" or `ftype`/`assoc` commands
- **Linux**: Create `.desktop` file in `~/.local/share/applications/`
- **macOS**: File associations typically require application bundles, not CLI tools

### Edge Cases Handled

**Empty VTT Files**

The tool generates an empty Markdown file (or empty output to stdout) and exits successfully. This is valid behavior—an empty transcript converts to an empty document.

**VTT Files with Only Metadata**

Files containing only NOTE, STYLE, or REGION blocks (no actual transcript cues) are treated as empty after parsing.

**Malformed or Unclosed Tags**

The parser handles malformed VTT tags gracefully:
- Unclosed `<v>` tags are recognized and processed
- Unknown or unmatched HTML tags are stripped
- Missing closing tags don't cause crashes

**Large Files**

The tool uses buffered reading (`BufRead` and `Lines` iterator) to process files incrementally. Multi-hour transcripts (several MB) are handled efficiently without excessive memory usage.

**Paths with Spaces**

The tool correctly handles file paths with spaces and special characters, both when invoked from the command line and when launched via file associations (where the OS may quote paths differently).

**Unicode and Emoji**

Speaker names containing Unicode characters (accented letters, emoji, etc.) are handled correctly. Unicode NFC normalization ensures consistent representation.

**HTML Entities**

The parser decodes common HTML character references:
- `&amp;` → `&`
- `&lt;` → `<`
- `&gt;` → `>`
- `&quot;` → `"`
- `&#39;` → `'`

**Out-of-Order Cues**

The parser sorts cues by timestamp after extraction. This handles Teams transcripts where cues may be interleaved or not strictly chronological in the file.

## Testing Guide

### How to Test This Tool

**Basic Conversion Test**

1. Create a simple test VTT file (`test.vtt`):
```
WEBVTT

00:00:01.000 --> 00:00:03.000
<v Alice>Hello world

00:00:03.500 --> 00:00:05.000
<v Bob>Hi Alice
```

2. Run the tool:
```bash
vtt-to-md test.vtt
```

3. Verify `test.md` is created with:
```markdown
**Alice:** Hello world

**Bob:** Hi Alice
```

**Teams Auto-Filtering Test**

1. Create a Teams-style VTT with unknown speakers:
```
WEBVTT

00:00:01.000 --> 00:00:03.000
<v @user123>Identified speaker

00:00:04.000 --> 00:00:05.000
No voice tag here

00:00:06.000 --> 00:00:08.000
<v Alice>Another identified speaker
```

2. Run without flags:
```bash
vtt-to-md test-teams.vtt
```

3. Verify output only contains identified speakers (unknown cue is filtered)

4. Run with `--no-filter-unknown`:
```bash
vtt-to-md test-teams.vtt --no-filter-unknown
```

5. Verify output now includes the "Unknown" speaker segment

**Multi-line Text Test**

1. Create a VTT with multi-line voice tags:
```
WEBVTT

00:00:01.000 --> 00:00:05.000
<v Alice>This is line one
and this is line two
and this is line three</v>
```

2. Run the tool and verify all three lines appear in the output

**Timestamp Test**

1. Run with `--include-timestamps first`:
```bash
vtt-to-md test.vtt --include-timestamps first
```

2. Verify timestamps appear at the beginning of each speaker turn: `[HH:MM:SS.mmm] **Speaker:**`

**Error Handling Test**

1. Test missing file:
```bash
vtt-to-md nonexistent.vtt
# Should exit with code 66 and print "file not found" error
```

2. Test output exists:
```bash
vtt-to-md test.vtt  # First run succeeds
vtt-to-md test.vtt  # Second run should fail with exit code 73
```

3. Test force overwrite:
```bash
vtt-to-md test.vtt --force
# Should succeed and overwrite existing file
```

4. Test conflicting flags:
```bash
vtt-to-md test.vtt --force --no-clobber
# Should fail with usage error (exit 64)
```

**Platform-Specific Tests**

Test with real VTT files from:
- **Microsoft Teams**: Verify `@` symbols are removed from speaker names, voice tags work, auto-filtering activates
- **Zoom**: Verify numeric speaker placeholders are handled
- **Google Meet**: Verify files with blank lines in cue payloads are handled correctly

### Automated Tests

The project includes comprehensive test coverage:
- **43 total tests** (30 unit tests + 13 integration tests)
- Unit tests cover: parsing, consolidation, markdown formatting, error types
- Integration tests cover: full conversion workflows, CLI flags, error conditions

Run tests with:
```bash
cargo test
```

Run with verbose output:
```bash
cargo test -- --nocapture
```

## Migration and Compatibility

### Breaking Changes

This is the initial release (v0.1.0), so there are no migration concerns. Future versions will maintain backward compatibility for CLI arguments and exit codes where possible.

### Platform Compatibility

The tool compiles and runs on:
- **Windows**: Tested on Windows 10/11 with PowerShell
- **Linux**: Tested on Ubuntu/Debian with bash
- **macOS**: Should work (untested, but uses standard Rust I/O)

Cross-platform binaries should be built on their respective platforms or via cross-compilation tools.

### Rust Version Requirements

- Minimum Rust version: Rust 2024 edition (Rust 1.80+)
- Specified via `rust-toolchain.toml` in the repository

## References

### Project Documentation
- Issue: https://github.com/lossyrob/vtt-to-md/issues/1
- Feature Specification: `.paw/work/vtt-to-md-cli/Spec.md`
- Implementation Plan: `.paw/work/vtt-to-md-cli/ImplementationPlan.md`

### External Standards
- **WebVTT Specification (W3C)**: https://www.w3.org/TR/webvtt1/
- **BSD sysexits.h**: https://man7.org/linux/man-pages/man3/sysexits.h.3head.html
- **Command Line Interface Guidelines**: https://clig.dev/
- **CommonMark Specification**: https://github.com/commonmark/commonmark-spec

### Rust Ecosystem
- **clap Documentation**: https://docs.rs/clap/
- **thiserror Documentation**: https://docs.rs/thiserror/
- **anyhow Documentation**: https://docs.rs/anyhow/
- **regex Documentation**: https://docs.rs/regex/
- **unicode-normalization Documentation**: https://docs.rs/unicode-normalization/
