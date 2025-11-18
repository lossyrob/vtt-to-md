# VTT to MD CLI Implementation Plan

## Overview

We're building a Rust command-line tool that converts WebVTT (Web Video Text Tracks) transcript files to readable Markdown format. The tool will parse VTT files downloaded from meeting platforms (Microsoft Teams, Zoom, Google Meet), extract speaker segments, consolidate consecutive utterances from the same speaker, and generate clean Markdown documents with bold speaker names followed by their text.

## Current State Analysis

The project is currently scaffolded with:
- `Cargo.toml`: Rust 2024 edition, version 0.1.0, no dependencies yet
- `src/main.rs`: Basic "Hello, world!" placeholder
- `README.md`: Basic build/run instructions

**Key Constraints Discovered:**
- Rust 2024 edition support is available
- No existing dependencies to migrate
- Clean slate for architecture decisions
- Cross-platform compilation required (Windows, Linux, macOS)

## Desired End State

A production-ready CLI executable that:
1. Accepts VTT file paths as arguments (including paths with spaces)
2. Parses WebVTT format robustly (handles Teams/Zoom/Meet variations)
3. Consolidates same-speaker text intelligently
4. Generates Markdown with format `**SpeakerName:** text`
5. Provides flags: `--force`, `--no-clobber`, `--stdout`, `--unknown-speaker`, `--include-timestamps`
6. Returns appropriate exit codes (0, 64, 65, 66, 77)
7. Handles errors gracefully with clear messages

**Verification**: Run `vtt-to-md meeting.vtt` on a Teams transcript and verify:
- Output file `meeting.md` created in same directory
- Speaker names are bold and sanitized
- Consecutive same-speaker cues are consolidated
- Exit code is 0
- Manual inspection confirms readability

## What We're NOT Doing

- GUI or interactive prompts
- Automatic file association registration (users configure manually)
- Multi-file batch processing in single invocation
- Exporting to formats other than Markdown (JSON, HTML, plain text)
- Speaker analytics (word count, speaking time)
- Integration with transcription services
- Preserving VTT STYLE/REGION blocks
- Supporting SRT or other subtitle formats
- Network operations or remote file fetching
- Configuration files or persistent settings
- Localization of error messages

## Implementation Approach

We'll build incrementally in six phases, each producing a testable artifact:

1. **Project Setup**: Configure dependencies, error types, exit codes
2. **CLI Parsing**: Implement argument/flag parsing with clap
3. **VTT Parser**: Build robust parser for WebVTT format with speaker extraction
4. **Consolidation**: Implement speaker-turn consolidation logic
5. **Markdown Generation**: Format and write output with proper escaping
6. **Testing**: Add unit tests, integration tests, and manual test cases

Each phase includes both automated verification (cargo commands) and manual verification (testing with real VTT files).

## Phase 1: Project Setup and Dependencies

### Overview
Configure Cargo.toml with required dependencies, define error types following sysexits conventions, and set up the module structure for clean separation of concerns.

### Changes Required:

#### 1. Update Cargo.toml
**File**: `Cargo.toml`
**Changes**: Add dependencies for CLI parsing, error handling, regex, and Unicode normalization

```toml
[package]
name = "vtt-to-md"
version = "0.1.0"
edition = "2024"

[dependencies]
clap = { version = "4.5", features = ["derive", "color", "suggestions"] }
anyhow = "1.0"
thiserror = "1.0"
regex = "1.10"
unicode-normalization = "0.1"
```

**Rationale**: 
- `clap` with derive feature for ergonomic CLI definition
- `anyhow` for easy error context chaining
- `thiserror` for defining custom error types
- `regex` for VTT pattern matching
- `unicode-normalization` for speaker name sanitization (NFC normalization)

#### 2. Create error module
**File**: `src/error.rs`
**Changes**: Define custom error types and exit code mapping

```rust
use std::process::ExitCode;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum VttError {
    #[error("Failed to read file: {path}")]
    FileRead { path: String, source: std::io::Error },
    
    #[error("Failed to write file: {path}")]
    FileWrite { path: String, source: std::io::Error },
    
    #[error("File not found: {path}")]
    FileNotFound { path: String },
    
    #[error("Permission denied: {path}")]
    PermissionDenied { path: String },
    
    #[error("Failed to parse VTT file: {reason}")]
    ParseError { reason: String },
    
    #[error("Invalid UTF-8 in file: {path}")]
    InvalidUtf8 { path: String },
    
    #[error("Output file already exists: {path} (use --force to overwrite)")]
    OutputExists { path: String },
    
    #[error("Cannot overwrite input file: {path}")]
    InputOutputSame { path: String },
}

impl VttError {
    pub fn exit_code(&self) -> ExitCode {
        match self {
            VttError::FileNotFound { .. } => ExitCode::from(66), // EX_NOINPUT
            VttError::PermissionDenied { .. } => ExitCode::from(77), // EX_NOPERM
            VttError::ParseError { .. } | VttError::InvalidUtf8 { .. } => ExitCode::from(65), // EX_DATAERR
            VttError::FileRead { .. } | VttError::FileWrite { .. } => ExitCode::from(74), // EX_IOERR
            VttError::OutputExists { .. } | VttError::InputOutputSame { .. } => ExitCode::from(73), // EX_CANTCREAT
        }
    }
}
```

**Rationale**: Following BSD sysexits.h conventions for cross-platform consistency. Each error variant maps to specific exit codes that scripts can handle programmatically.

#### 3. Update main.rs module structure
**File**: `src/main.rs`
**Changes**: Set up module declarations and main function skeleton

```rust
mod error;
mod cli;
mod parser;
mod consolidator;
mod markdown;

use std::process::ExitCode;
use error::VttError;

fn main() -> ExitCode {
    // CLI parsing will be added in Phase 2
    println!("VTT to MD converter - Phase 1 complete");
    ExitCode::SUCCESS
}
```

#### 4. Create module files
**Files**: Create empty module files for future phases
- `src/cli.rs` (Phase 2)
- `src/parser.rs` (Phase 3)
- `src/consolidator.rs` (Phase 4)
- `src/markdown.rs` (Phase 5)

```rust
// src/cli.rs
// CLI parsing module - to be implemented in Phase 2

// src/parser.rs
// VTT parsing module - to be implemented in Phase 3

// src/consolidator.rs
// Speaker consolidation module - to be implemented in Phase 4

// src/markdown.rs
// Markdown generation module - to be implemented in Phase 5
```

### Success Criteria:

#### Automated Verification:
- [ ] Project builds successfully: `cargo build`
- [ ] Dependencies resolve without conflicts: `cargo check`
- [ ] No compiler warnings: `cargo clippy`
- [ ] Code formatting passes: `cargo fmt --check`

#### Manual Verification:
- [ ] Cargo.toml lists all required dependencies with correct versions
- [ ] error.rs defines all error variants from Spec.md requirements
- [ ] Exit codes match sysexits.h conventions (64, 65, 66, 73, 74, 77)
- [ ] Module structure is clear and follows Rust conventions

---

## Phase 2: CLI Argument Parsing

### Overview
Implement command-line interface using clap with derive macros. Handle positional arguments for input/output paths and flags for behavior control (--force, --no-clobber, --stdout, --unknown-speaker, --include-timestamps).

### Changes Required:

#### 1. Define CLI structure with clap
**File**: `src/cli.rs`
**Changes**: Create Args struct with clap derive macros

```rust
use clap::{Parser, ValueEnum};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(name = "vtt-to-md")]
#[command(about = "Convert WebVTT transcript files to Markdown format", long_about = None)]
#[command(version)]
pub struct Args {
    /// Input VTT file path
    #[arg(value_name = "INPUT")]
    pub input: PathBuf,

    /// Output Markdown file path (defaults to input name with .md extension)
    #[arg(value_name = "OUTPUT")]
    pub output: Option<PathBuf>,

    /// Overwrite output file if it exists
    #[arg(short, long)]
    pub force: bool,

    /// Skip conversion if output file exists
    #[arg(short = 'n', long)]
    pub no_clobber: bool,

    /// Print output to stdout instead of writing to file
    #[arg(long)]
    pub stdout: bool,

    /// Label to use for cues without speaker attribution
    #[arg(long, default_value = "Unknown")]
    pub unknown_speaker: String,

    /// Include timestamps in output
    #[arg(long, value_enum, default_value = "none")]
    pub include_timestamps: TimestampMode,
}

#[derive(Debug, Clone, ValueEnum)]
pub enum TimestampMode {
    /// No timestamps in output
    None,
    /// First timestamp of each speaker turn
    First,
    /// Timestamp for each utterance
    Each,
}

impl Args {
    pub fn parse_args() -> Self {
        Args::parse()
    }

    /// Get the output path, deriving from input if not specified
    pub fn get_output_path(&self) -> Option<PathBuf> {
        if self.stdout {
            return None;
        }

        if let Some(ref output) = self.output {
            Some(output.clone())
        } else {
            // Derive from input: replace extension with .md
            let mut output = self.input.clone();
            output.set_extension("md");
            Some(output)
        }
    }

    /// Validate that input and output are not the same file
    pub fn validate(&self) -> Result<(), crate::error::VttError> {
        use crate::error::VttError;

        // Check for conflicting flags
        if self.force && self.no_clobber {
            return Err(VttError::ParseError {
                reason: "Cannot use --force and --no-clobber together".to_string(),
            });
        }

        // Check that output is not same as input
        if let Some(output) = self.get_output_path() {
            if self.input.canonicalize().ok() == output.canonicalize().ok()
                && self.input.exists()
            {
                return Err(VttError::InputOutputSame {
                    path: self.input.display().to_string(),
                });
            }
        }

        Ok(())
    }
}
```

#### 2. Integrate CLI parsing into main
**File**: `src/main.rs`
**Changes**: Parse arguments and validate before processing

```rust
mod error;
mod cli;
mod parser;
mod consolidator;
mod markdown;

use std::process::ExitCode;
use error::VttError;
use cli::Args;

fn main() -> ExitCode {
    let args = match std::panic::catch_unwind(|| Args::parse_args()) {
        Ok(args) => args,
        Err(_) => {
            // Clap handles its own error messages and exits
            return ExitCode::from(64); // EX_USAGE
        }
    };

    if let Err(e) = args.validate() {
        eprintln!("Error: {}", e);
        return e.exit_code();
    }

    println!("Input: {:?}", args.input);
    println!("Output: {:?}", args.get_output_path());
    println!("Phase 2 complete");
    
    ExitCode::SUCCESS
}
```

### Success Criteria:

#### Automated Verification:
- [ ] Project compiles: `cargo build`
- [ ] CLI help displays correctly: `cargo run -- --help`
- [ ] Version flag works: `cargo run -- --version`
- [ ] Invalid args show usage: `cargo run -- --invalid` (exit code 64)
- [ ] Conflicting flags detected: `cargo run -- input.vtt --force --no-clobber` (exit code 65)

#### Manual Verification:
- [ ] Running with file path shows correct input/output derivation
- [ ] Path with spaces handled: `cargo run -- "my file.vtt"`
- [ ] Custom output path accepted: `cargo run -- input.vtt output.md`
- [ ] Flags parse correctly: `cargo run -- input.vtt --force --stdout --unknown-speaker "Narrator"`
- [ ] Help text is clear and describes all options

---

## Phase 3: VTT Parser Implementation

### Overview
Build a robust parser that reads VTT files, extracts cue text and speaker attributions from `<v>` tags, handles HTML character references, and gracefully manages malformed syntax variations from different platforms.

### Changes Required:

#### 1. Define data structures for parsed VTT
**File**: `src/parser.rs`
**Changes**: Create types for VTT cues and speaker segments

```rust
use crate::error::VttError;
use regex::Regex;
use std::fs;
use std::path::Path;

#[derive(Debug, Clone)]
pub struct Cue {
    pub timestamp: Option<String>,
    pub speaker: Option<String>,
    pub text: String,
}

#[derive(Debug)]
pub struct VttDocument {
    pub cues: Vec<Cue>,
}

impl VttDocument {
    pub fn parse_file(path: &Path) -> Result<Self, VttError> {
        let content = fs::read_to_string(path).map_err(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                VttError::FileNotFound {
                    path: path.display().to_string(),
                }
            } else if e.kind() == std::io::ErrorKind::PermissionDenied {
                VttError::PermissionDenied {
                    path: path.display().to_string(),
                }
            } else {
                VttError::FileRead {
                    path: path.display().to_string(),
                    source: e,
                }
            }
        })?;

        Self::parse_content(&content, path)
    }

    fn parse_content(content: &str, path: &Path) -> Result<Self, VttError> {
        // Validate VTT header
        if !content.starts_with("WEBVTT") {
            return Err(VttError::ParseError {
                reason: format!("File does not start with WEBVTT header: {}", path.display()),
            });
        }

        let cues = Self::extract_cues(content, path)?;
        Ok(VttDocument { cues })
    }

    fn extract_cues(content: &str, path: &Path) -> Result<Vec<Cue>, VttError> {
        let mut cues = Vec::new();

        // Regex for voice tags: <v SpeakerName>text</v>
        let voice_regex = Regex::new(r"<v\s+([^>]+)>(.*?)</v>").unwrap();
        
        // Regex for timestamps: 00:00:00.000 --> 00:00:05.000
        let timestamp_regex = Regex::new(
            r"(\d{2}:\d{2}:\d{2}\.\d{3})\s+-->\s+(\d{2}:\d{2}:\d{2}\.\d{3})"
        ).unwrap();

        let lines: Vec<&str> = content.lines().collect();
        let mut i = 0;

        while i < lines.len() {
            let line = lines[i].trim();

            // Skip empty lines, NOTE blocks, STYLE blocks, REGION blocks
            if line.is_empty() 
                || line.starts_with("NOTE") 
                || line.starts_with("STYLE") 
                || line.starts_with("REGION")
                || line.starts_with("WEBVTT") {
                i += 1;
                continue;
            }

            // Look for timestamp line
            if let Some(ts_match) = timestamp_regex.captures(line) {
                let timestamp = ts_match.get(1).map(|m| m.as_str().to_string());
                
                // Collect cue text (lines until next empty line or timestamp)
                i += 1;
                let mut cue_text = String::new();
                
                while i < lines.len() {
                    let text_line = lines[i].trim();
                    
                    // Stop at empty line or next timestamp
                    if text_line.is_empty() || timestamp_regex.is_match(text_line) {
                        break;
                    }
                    
                    if !cue_text.is_empty() {
                        cue_text.push(' ');
                    }
                    cue_text.push_str(text_line);
                    i += 1;
                }

                // Extract speaker and text from voice tags
                if let Some(cap) = voice_regex.captures(&cue_text) {
                    let speaker = cap.get(1).map(|m| m.as_str().to_string());
                    let text = cap.get(2).map(|m| m.as_str()).unwrap_or("");
                    
                    let sanitized_speaker = speaker.map(|s| Self::sanitize_speaker(&s));
                    let cleaned_text = Self::clean_text(text);

                    if !cleaned_text.is_empty() {
                        cues.push(Cue {
                            timestamp,
                            speaker: sanitized_speaker,
                            text: cleaned_text,
                        });
                    }
                } else {
                    // No voice tag - treat as unknown speaker
                    let cleaned_text = Self::clean_text(&cue_text);
                    if !cleaned_text.is_empty() {
                        cues.push(Cue {
                            timestamp,
                            speaker: None,
                            text: cleaned_text,
                        });
                    }
                }
            } else {
                // Not a timestamp line, skip
                i += 1;
            }
        }

        if cues.is_empty() {
            return Err(VttError::ParseError {
                reason: format!("No valid cues found in file: {}", path.display()),
            });
        }

        Ok(cues)
    }

    fn sanitize_speaker(speaker: &str) -> String {
        use unicode_normalization::UnicodeNormalization;

        let mut sanitized = speaker.trim().to_string();

        // Remove @ symbols
        sanitized = sanitized.replace('@', "");

        // Apply Unicode NFC normalization
        sanitized = sanitized.nfc().collect();

        // Escape Markdown special characters
        for ch in ['*', '_', '#', '[', ']', '(', ')', '{', '}', '!', '>', '|', '`', '\\'] {
            sanitized = sanitized.replace(ch, &format!("\\{}", ch));
        }

        sanitized.trim().to_string()
    }

    fn clean_text(text: &str) -> String {
        // Strip HTML tags (except voice tags which are already handled)
        let tag_regex = Regex::new(r"<[^>]+>").unwrap();
        let mut cleaned = tag_regex.replace_all(text, "").to_string();

        // Decode HTML character references
        cleaned = cleaned
            .replace("&amp;", "&")
            .replace("&lt;", "<")
            .replace("&gt;", ">")
            .replace("&quot;", "\"")
            .replace("&#39;", "'")
            .replace("&apos;", "'");

        // Trim and normalize whitespace
        cleaned.split_whitespace().collect::<Vec<_>>().join(" ")
    }
}
```

### Success Criteria:

#### Automated Verification:
- [ ] Code compiles: `cargo build`
- [ ] Unit test for valid VTT file passes
- [ ] Unit test for missing WEBVTT header fails correctly
- [ ] Unit test for speaker sanitization works (removes @, escapes *)
- [ ] Unit test for HTML entity decoding works (&amp; → &)

#### Manual Verification:
- [ ] Parse a real Teams VTT file with `<v>` tags successfully
- [ ] Parse a file with missing speaker tags (Unknown speaker assigned)
- [ ] Parse a file with malformed tags without crashing
- [ ] Verify speaker names with @ symbols are cleaned
- [ ] Verify HTML entities in text are decoded correctly

---

## Phase 4: Speaker Consolidation

### Overview
Implement logic to consolidate consecutive cues from the same speaker into single segments, joining text with appropriate whitespace while respecting sentence boundaries.

### Changes Required:

#### 1. Create consolidator module
**File**: `src/consolidator.rs`
**Changes**: Define consolidation logic for speaker segments

```rust
use crate::parser::{Cue, VttDocument};
use crate::cli::TimestampMode;

#[derive(Debug)]
pub struct SpeakerSegment {
    pub speaker: String,
    pub text: String,
    pub timestamp: Option<String>,
}

pub struct Consolidator {
    unknown_speaker_label: String,
    timestamp_mode: TimestampMode,
}

impl Consolidator {
    pub fn new(unknown_speaker_label: String, timestamp_mode: TimestampMode) -> Self {
        Consolidator {
            unknown_speaker_label,
            timestamp_mode,
        }
    }

    pub fn consolidate(&self, document: &VttDocument) -> Vec<SpeakerSegment> {
        if document.cues.is_empty() {
            return Vec::new();
        }

        let mut segments = Vec::new();
        let mut current_speaker: Option<String> = None;
        let mut current_text = String::new();
        let mut current_timestamp: Option<String> = None;

        for cue in &document.cues {
            let speaker = cue.speaker.clone()
                .unwrap_or_else(|| self.unknown_speaker_label.clone());

            // Check if speaker changed
            if Some(&speaker) != current_speaker.as_ref() {
                // Save previous segment if exists
                if let Some(prev_speaker) = current_speaker.take() {
                    if !current_text.trim().is_empty() {
                        segments.push(SpeakerSegment {
                            speaker: prev_speaker,
                            text: current_text.trim().to_string(),
                            timestamp: current_timestamp.clone(),
                        });
                    }
                }

                // Start new segment
                current_speaker = Some(speaker);
                current_text = cue.text.clone();
                current_timestamp = cue.timestamp.clone();
            } else {
                // Same speaker - consolidate text
                if !current_text.is_empty() {
                    // Add space between cues
                    // If previous text ends with terminal punctuation, just add space
                    // Otherwise, join with space
                    let ends_with_terminal = current_text.trim_end()
                        .ends_with(|c: char| c == '.' || c == '?' || c == '!');
                    
                    if ends_with_terminal {
                        current_text.push(' ');
                    } else {
                        current_text.push(' ');
                    }
                }
                current_text.push_str(&cue.text);
            }
        }

        // Don't forget the last segment
        if let Some(speaker) = current_speaker {
            if !current_text.trim().is_empty() {
                segments.push(SpeakerSegment {
                    speaker,
                    text: current_text.trim().to_string(),
                    timestamp: current_timestamp,
                });
            }
        }

        segments
    }
}
```

### Success Criteria:

#### Automated Verification:
- [ ] Code compiles: `cargo build`
- [ ] Unit test: consecutive same-speaker cues consolidate into one segment
- [ ] Unit test: different speakers create separate segments
- [ ] Unit test: empty/whitespace-only cues are skipped
- [ ] Unit test: unknown speaker label is applied correctly

#### Manual Verification:
- [ ] Test with VTT file containing 3 consecutive cues from same speaker
- [ ] Verify output has single segment with all text joined
- [ ] Test with alternating speakers (A, B, A, B pattern)
- [ ] Verify output has 4 separate segments in correct order
- [ ] Check text joining respects sentence boundaries (no missing spaces)

---

## Phase 5: Markdown Generation and File Output

### Overview
Generate Markdown output with bold speaker names, handle file writing with existence checks, implement --force/--no-clobber/--stdout flags, and ensure proper error handling for all file operations.

### Changes Required:

#### 1. Create markdown generator module
**File**: `src/markdown.rs`
**Changes**: Format segments as Markdown and handle file output

```rust
use crate::consolidator::SpeakerSegment;
use crate::error::VttError;
use crate::cli::TimestampMode;
use std::fs;
use std::io::Write;
use std::path::Path;

pub struct MarkdownGenerator {
    timestamp_mode: TimestampMode,
}

impl MarkdownGenerator {
    pub fn new(timestamp_mode: TimestampMode) -> Self {
        MarkdownGenerator { timestamp_mode }
    }

    pub fn generate(&self, segments: &[SpeakerSegment]) -> String {
        let mut markdown = String::new();

        for segment in segments {
            // Add timestamp if requested
            let prefix = match self.timestamp_mode {
                TimestampMode::None => String::new(),
                TimestampMode::First | TimestampMode::Each => {
                    if let Some(ref ts) = segment.timestamp {
                        format!("[{}] ", ts)
                    } else {
                        String::new()
                    }
                }
            };

            // Format as: **Speaker:** text
            markdown.push_str(&format!(
                "{}**{}:** {}\n\n",
                prefix,
                segment.speaker,
                segment.text
            ));
        }

        markdown
    }

    pub fn write_to_file(&self, content: &str, path: &Path, force: bool) -> Result<(), VttError> {
        // Check if file exists
        if path.exists() && !force {
            return Err(VttError::OutputExists {
                path: path.display().to_string(),
            });
        }

        fs::write(path, content).map_err(|e| {
            if e.kind() == std::io::ErrorKind::PermissionDenied {
                VttError::PermissionDenied {
                    path: path.display().to_string(),
                }
            } else {
                VttError::FileWrite {
                    path: path.display().to_string(),
                    source: e,
                }
            }
        })
    }

    pub fn write_to_stdout(&self, content: &str) -> Result<(), VttError> {
        std::io::stdout()
            .write_all(content.as_bytes())
            .map_err(|e| VttError::FileWrite {
                path: "stdout".to_string(),
                source: e,
            })
    }
}
```

#### 2. Integrate all phases in main
**File**: `src/main.rs`
**Changes**: Wire up all modules and handle complete conversion flow

```rust
mod error;
mod cli;
mod parser;
mod consolidator;
mod markdown;

use std::process::ExitCode;
use error::VttError;
use cli::Args;
use parser::VttDocument;
use consolidator::Consolidator;
use markdown::MarkdownGenerator;

fn main() -> ExitCode {
    let args = match std::panic::catch_unwind(|| Args::parse_args()) {
        Ok(args) => args,
        Err(_) => {
            return ExitCode::from(64); // EX_USAGE
        }
    };

    if let Err(e) = run(args) {
        eprintln!("Error: {}", e);
        return e.exit_code();
    }

    ExitCode::SUCCESS
}

fn run(args: Args) -> Result<(), VttError> {
    // Validate arguments
    args.validate()?;

    // Parse VTT file
    let document = VttDocument::parse_file(&args.input)?;

    // Consolidate speaker segments
    let consolidator = Consolidator::new(
        args.unknown_speaker.clone(),
        args.include_timestamps.clone(),
    );
    let segments = consolidator.consolidate(&document);

    // Generate Markdown
    let generator = MarkdownGenerator::new(args.include_timestamps.clone());
    let markdown = generator.generate(&segments);

    // Output handling
    if args.stdout {
        // Print to stdout
        generator.write_to_stdout(&markdown)?;
    } else {
        let output_path = args.get_output_path()
            .expect("Output path should exist when not using stdout");

        // Check --no-clobber flag
        if args.no_clobber && output_path.exists() {
            eprintln!("Output file exists, skipping: {}", output_path.display());
            return Ok(());
        }

        // Write to file
        generator.write_to_file(&markdown, &output_path, args.force)?;
        
        println!("Converted {} -> {}", 
            args.input.display(), 
            output_path.display());
    }

    Ok(())
}
```

### Success Criteria:

#### Automated Verification:
- [ ] Full build succeeds: `cargo build --release`
- [ ] Basic conversion works: Create test.vtt, run `cargo run -- test.vtt`, verify test.md exists
- [ ] Force flag works: Run twice with --force, verify no error
- [ ] No-clobber works: Run twice with --no-clobber, verify skip message
- [ ] Stdout works: `cargo run -- test.vtt --stdout` prints Markdown
- [ ] Clippy passes: `cargo clippy -- -D warnings`
- [ ] Format check passes: `cargo fmt --check`

#### Manual Verification:
- [ ] Convert a real Teams VTT file and inspect Markdown quality
- [ ] Verify speaker names are bold and properly formatted
- [ ] Verify consecutive same-speaker text is consolidated
- [ ] Test with Zoom VTT file (different format variations)
- [ ] Test with Google Meet VTT file (blank lines, missing tags)
- [ ] Test error cases: nonexistent file (exit 66), permission denied
- [ ] Verify output file is in same directory as input
- [ ] Test file path with spaces works correctly

---

## Phase 6: Testing Strategy

### Overview
Add comprehensive unit tests for each module, integration tests for end-to-end conversion scenarios, and document manual test cases for platform-specific VTT variations.

### Changes Required:

#### 1. Create unit tests for parser
**File**: `src/parser.rs`
**Changes**: Add test module at end of file

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_parse_valid_vtt() {
        let content = r#"WEBVTT

00:00:00.000 --> 00:00:05.000
<v Alice>Hello, world!</v>

00:00:05.000 --> 00:00:10.000
<v Bob>Hi there!</v>
"#;
        let mut file = NamedTempFile::new().unwrap();
        file.write_all(content.as_bytes()).unwrap();
        
        let doc = VttDocument::parse_file(file.path()).unwrap();
        assert_eq!(doc.cues.len(), 2);
        assert_eq!(doc.cues[0].speaker.as_deref(), Some("Alice"));
        assert_eq!(doc.cues[0].text, "Hello, world!");
    }

    #[test]
    fn test_sanitize_speaker_removes_at_symbol() {
        let result = VttDocument::sanitize_speaker("@user1234");
        assert_eq!(result, "user1234");
    }

    #[test]
    fn test_sanitize_speaker_escapes_markdown() {
        let result = VttDocument::sanitize_speaker("Name*With_Special#Chars");
        assert!(result.contains("\\*"));
        assert!(result.contains("\\_"));
        assert!(result.contains("\\#"));
    }

    #[test]
    fn test_clean_text_decodes_html_entities() {
        let result = VttDocument::clean_text("Hello &amp; goodbye");
        assert_eq!(result, "Hello & goodbye");
    }

    #[test]
    fn test_parse_missing_header() {
        let content = "Not a VTT file";
        let mut file = NamedTempFile::new().unwrap();
        file.write_all(content.as_bytes()).unwrap();
        
        let result = VttDocument::parse_file(file.path());
        assert!(result.is_err());
    }
}
```

#### 2. Create unit tests for consolidator
**File**: `src/consolidator.rs`
**Changes**: Add test module

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::TimestampMode;

    #[test]
    fn test_consolidate_same_speaker() {
        let cues = vec![
            Cue {
                timestamp: Some("00:00:00.000".to_string()),
                speaker: Some("Alice".to_string()),
                text: "Hello".to_string(),
            },
            Cue {
                timestamp: Some("00:00:05.000".to_string()),
                speaker: Some("Alice".to_string()),
                text: "world!".to_string(),
            },
        ];
        
        let doc = VttDocument { cues };
        let consolidator = Consolidator::new("Unknown".to_string(), TimestampMode::None);
        let segments = consolidator.consolidate(&doc);
        
        assert_eq!(segments.len(), 1);
        assert_eq!(segments[0].speaker, "Alice");
        assert_eq!(segments[0].text, "Hello world!");
    }

    #[test]
    fn test_consolidate_different_speakers() {
        let cues = vec![
            Cue {
                timestamp: Some("00:00:00.000".to_string()),
                speaker: Some("Alice".to_string()),
                text: "Hello".to_string(),
            },
            Cue {
                timestamp: Some("00:00:05.000".to_string()),
                speaker: Some("Bob".to_string()),
                text: "Hi there".to_string(),
            },
        ];
        
        let doc = VttDocument { cues };
        let consolidator = Consolidator::new("Unknown".to_string(), TimestampMode::None);
        let segments = consolidator.consolidate(&doc);
        
        assert_eq!(segments.len(), 2);
        assert_eq!(segments[0].speaker, "Alice");
        assert_eq!(segments[1].speaker, "Bob");
    }

    #[test]
    fn test_unknown_speaker_label() {
        let cues = vec![
            Cue {
                timestamp: Some("00:00:00.000".to_string()),
                speaker: None,
                text: "Mystery text".to_string(),
            },
        ];
        
        let doc = VttDocument { cues };
        let consolidator = Consolidator::new("Narrator".to_string(), TimestampMode::None);
        let segments = consolidator.consolidate(&doc);
        
        assert_eq!(segments.len(), 1);
        assert_eq!(segments[0].speaker, "Narrator");
    }
}
```

#### 3. Create integration tests
**File**: `tests/integration_test.rs`
**Changes**: Create end-to-end test cases

```rust
use std::fs;
use std::process::Command;
use tempfile::TempDir;

#[test]
fn test_basic_conversion() {
    let temp_dir = TempDir::new().unwrap();
    let input_path = temp_dir.path().join("test.vtt");
    let output_path = temp_dir.path().join("test.md");

    let vtt_content = r#"WEBVTT

00:00:00.000 --> 00:00:05.000
<v Alice>Hello, everyone!</v>

00:00:05.000 --> 00:00:10.000
<v Bob>Hi Alice!</v>
"#;

    fs::write(&input_path, vtt_content).unwrap();

    let status = Command::new("cargo")
        .args(["run", "--", input_path.to_str().unwrap()])
        .status()
        .unwrap();

    assert!(status.success());
    assert!(output_path.exists());

    let output_content = fs::read_to_string(&output_path).unwrap();
    assert!(output_content.contains("**Alice:** Hello, everyone!"));
    assert!(output_content.contains("**Bob:** Hi Alice!"));
}

#[test]
fn test_force_overwrite() {
    let temp_dir = TempDir::new().unwrap();
    let input_path = temp_dir.path().join("test.vtt");
    let output_path = temp_dir.path().join("test.md");

    let vtt_content = r#"WEBVTT

00:00:00.000 --> 00:00:05.000
<v Alice>Test</v>
"#;

    fs::write(&input_path, vtt_content).unwrap();
    fs::write(&output_path, "Old content").unwrap();

    let status = Command::new("cargo")
        .args(["run", "--", input_path.to_str().unwrap(), "--force"])
        .status()
        .unwrap();

    assert!(status.success());
    let output_content = fs::read_to_string(&output_path).unwrap();
    assert!(output_content.contains("**Alice:** Test"));
}

#[test]
fn test_file_not_found() {
    let status = Command::new("cargo")
        .args(["run", "--", "nonexistent.vtt"])
        .status()
        .unwrap();

    assert_eq!(status.code(), Some(66));
}
```

#### 4. Add test dependency to Cargo.toml
**File**: `Cargo.toml`
**Changes**: Add dev-dependencies section

```toml
[dev-dependencies]
tempfile = "3.8"
```

#### 5. Document manual test cases
**File**: `tests/MANUAL_TESTS.md`
**Changes**: Create manual testing checklist

```markdown
# Manual Test Cases for VTT to MD

## Platform-Specific VTT Files

### Microsoft Teams
1. Download a Teams meeting transcript (.vtt)
2. Run: `vtt-to-md teams-meeting.vtt`
3. Verify:
   - [ ] Speaker names extracted correctly from `<v>` tags
   - [ ] Consecutive same-speaker segments consolidated
   - [ ] @ symbols removed from anonymized speakers
   - [ ] Output is readable and properly formatted

### Zoom
1. Download a Zoom cloud recording transcript
2. Run: `vtt-to-md zoom-recording.vtt`
3. Verify:
   - [ ] Numeric speaker placeholders handled (if present)
   - [ ] Speaker names with special characters work
   - [ ] Timestamps parsed correctly

### Google Meet
1. Export a Meet transcript
2. Run: `vtt-to-md meet-transcript.vtt`
3. Verify:
   - [ ] Files with blank lines in cue payloads work
   - [ ] Missing `<v>` tags default to Unknown speaker
   - [ ] Non-standard formatting doesn't crash parser

## File Association Testing

### Windows
1. Compile release: `cargo build --release`
2. Right-click .vtt file → Open With → Choose vtt-to-md.exe
3. Verify: .md file created in same directory

### Linux
1. Create .desktop entry pointing to executable
2. Double-click .vtt file in file manager
3. Verify: .md file created

### macOS
1. Note: CLI tools typically invoked via Terminal or scripts
2. Test: `open -a Terminal file.vtt` (if configured)

## Edge Cases

### Large Files
1. Create/download multi-hour meeting transcript (several MB)
2. Run: `vtt-to-md large-meeting.vtt`
3. Verify:
   - [ ] Completes in reasonable time (< 5 seconds)
   - [ ] No memory issues
   - [ ] Output file is correct

### Special Characters
1. Test file with paths containing spaces: `"my file.vtt"`
2. Test speaker names with emoji, Unicode
3. Test text with HTML entities (&amp;, &lt;, etc.)

### Error Conditions
1. Nonexistent file: verify exit code 66
2. Read-protected file: verify exit code 77
3. Write-protected directory: verify exit code 77
4. Invalid VTT format: verify exit code 65
```

### Success Criteria:

#### Automated Verification:
- [ ] All unit tests pass: `cargo test`
- [ ] All integration tests pass: `cargo test --test integration_test`
- [ ] Code coverage > 80% (if coverage tool configured)
- [ ] No test failures on Windows, Linux, macOS (CI)

#### Manual Verification:
- [ ] Complete all test cases in MANUAL_TESTS.md
- [ ] Test with real VTT files from Teams, Zoom, Meet
- [ ] Verify file association works on at least one platform
- [ ] Test edge cases: large files, special characters, errors
- [ ] Verify error messages are clear and actionable

---

## Testing Strategy

### Unit Tests
Focus on individual components in isolation:
- **Parser**: VTT format parsing, speaker extraction, text cleaning, HTML decoding
- **Consolidator**: Same-speaker merging, different-speaker separation, unknown speaker handling
- **Markdown**: Format generation, timestamp inclusion modes
- **Error handling**: Exit code mapping, error message generation

### Integration Tests
Test complete conversion workflows:
- Basic VTT to MD conversion
- Output file placement (same directory, correct name)
- Flag behavior: --force, --no-clobber, --stdout
- Error scenarios: missing file, permission errors
- Custom unknown speaker label
- Timestamp inclusion modes

### Manual Testing
Real-world validation:
- Platform-specific VTT files (Teams, Zoom, Meet)
- File association workflows (double-click)
- Large files (performance)
- Special characters in paths and content
- Cross-platform compatibility

### Test Data
Create minimal test VTT files for:
- Simple two-speaker conversation
- Same speaker consecutive cues
- Missing voice tags
- Malformed tags
- HTML entities
- Empty file
- No WEBVTT header

---

## Performance Considerations

The tool processes files incrementally using buffered reading (`BufRead`) to handle large transcripts efficiently. Typical meeting transcripts are:
- 1 hour meeting: ~80-400 KB
- Multi-hour meeting: few MB at most

Expected performance: < 1 second for typical files, < 5 seconds for multi-hour transcripts on modern hardware.

No optimization needed initially; profile if performance issues arise with real-world usage.

---

## Migration Notes

N/A - This is a new tool with no existing users or data to migrate.

---

## References

- Original Issue: https://github.com/lossyrob/vtt-to-md/issues/1
- Spec: `.paw/work/vtt-to-md-cli/Spec.md`
- Research: `.paw/work/vtt-to-md-cli/SpecResearch.md`
- WebVTT Specification: https://www.w3.org/TR/webvtt1/
- BSD sysexits: https://man7.org/linux/man-pages/man3/sysexits.h.3head.html
- Command Line Interface Guidelines: https://clig.dev/
- CommonMark Specification: https://github.com/commonmark/commonmark-spec
