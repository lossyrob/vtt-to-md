# Feature Specification: VTT to MD CLI

**Branch**: feature/v1  |  **Created**: 2025-11-17  |  **Status**: Draft
**Input Brief**: Convert WebVTT transcript files to readable Markdown format via standalone CLI executable

## Overview

Meeting transcripts saved as WebVTT files contain valuable conversation content, but their technical format makes them difficult to read, search, and share. A user downloads a transcript from Microsoft Teams, Zoom, or Google Meet and finds a `.vtt` file filled with timestamps and formatting tags. They want to quickly transform this into a clean, readable Markdown document that preserves speaker attribution and conversation flow. This CLI tool bridges that gap by parsing the WebVTT structure, extracting speaker segments, consolidating consecutive utterances from the same speaker, and generating a well-formatted Markdown transcript.

The tool operates as a simple command-line executable that accepts a VTT file path and produces a corresponding Markdown file in the same directory. Users invoke it directly from their terminal or, after configuring file associations, simply double-click a VTT file to trigger the conversion automatically. The generated Markdown uses bold speaker names followed by their consolidated text, creating a natural reading experience that mirrors how transcripts are typically formatted in meeting notes or documentation. By handling edge cases like missing speaker names, malformed VTT syntax, and platform-specific variations (Teams' `<v>` tags, Meet's non-standard formatting), the tool ensures reliable output across different transcript sources.

Beyond basic conversion, the tool provides thoughtful defaults and optional controls. It automatically handles speaker name sanitization (removing @ symbols for anonymized users, escaping Markdown special characters), consolidates whitespace intelligently (joining same-speaker cues while respecting sentence boundaries), and offers flags for advanced use cases like printing to stdout for piping, forcing overwrites of existing files, or customizing unknown speaker labels. The result is a focused, reliable utility that solves a real workflow pain point: transforming machine-generated transcripts into human-readable documents suitable for archiving, sharing, or further editing.

## Objectives

- Enable one-command conversion from WebVTT transcript files to readable Markdown documents
- Extract and preserve speaker attribution from VTT voice tags, with graceful handling of missing or malformed speaker data
- Consolidate consecutive utterances from the same speaker into coherent paragraphs
- Support file association workflows where double-clicking VTT files triggers automatic conversion
- Generate Markdown output alongside the input file (same directory, `.md` extension) with configurable overwrite behavior
- Handle real-world transcript variations from Microsoft Teams, Zoom, and Google Meet (Rationale: users download transcripts from multiple platforms with inconsistent formatting)
- Provide cross-platform executable with no runtime dependencies beyond the OS shell (Rationale: simplifies deployment and enables file association on Windows, Linux, and macOS)
- Surface clear, actionable error messages for common failure scenarios (file not found, permission denied, parse errors) with appropriate exit codes
- Offer optional flags for advanced workflows (stdout output for piping, custom unknown speaker labels, overwrite control)

## User Scenarios & Testing

### User Story P1 – Convert VTT file to Markdown via CLI

**Narrative**: A user with a downloaded VTT transcript runs the CLI tool, providing the VTT file path as an argument. The tool parses the VTT content, extracts speaker segments, consolidates same-speaker text, and writes a Markdown file to the same directory with the same base name and `.md` extension. The user opens the Markdown file and sees a clean, readable transcript with speaker names in bold followed by their consolidated text.

**Independent Test**: Run `vtt-to-md input.vtt` on a valid VTT file with multiple speakers and verify that `input.md` is created in the same directory with correct Markdown formatting.

**Acceptance Scenarios**:
1. Given a VTT file `meeting.vtt` containing speaker segments with `<v SpeakerName>` tags, When user runs `vtt-to-md meeting.vtt`, Then `meeting.md` is created in the same directory with speaker names in bold and consolidated text
2. Given a VTT file with consecutive cues from the same speaker, When the tool processes it, Then the output consolidates these cues into a single paragraph per speaker turn without duplicate speaker labels
3. Given a VTT file in a directory where the user has write permissions, When conversion succeeds, Then the tool exits with code 0 and the output file exists
4. Given a VTT file with speaker names containing @ symbols (e.g., `@user1234`), When conversion runs, Then the output Markdown shows sanitized speaker names with @ symbols removed
5. Given a VTT file with speaker names containing Markdown special characters (e.g., `*`, `_`, `#`), When conversion runs, Then the output properly escapes these characters to prevent unintended formatting

### User Story P2 – Handle missing or unknown speakers

**Narrative**: A user converts a VTT file where some cues lack `<v>` tags or have empty speaker annotations. The tool assigns a fallback label (e.g., "Unknown") to these segments and consolidates them separately from identified speakers, ensuring the output remains structured and readable even when speaker attribution is incomplete.

**Independent Test**: Run the tool on a VTT file containing cues without `<v>` tags and verify that these cues appear under a default "Unknown" speaker label in the output Markdown.

**Acceptance Scenarios**:
1. Given a VTT file with some cues missing `<v>` tags, When conversion runs, Then unattributed text appears under a default speaker label (e.g., "Unknown")
2. Given a VTT file with empty `<v>` annotations (e.g., `<v>`), When conversion runs, Then these cues are treated as unknown speaker segments
3. Given a VTT file where multiple separate segments lack speaker attribution, When conversion runs, Then these segments are not incorrectly consolidated unless they are consecutive

### User Story P3 – Double-click VTT file for automatic conversion

**Narrative**: A user has configured their operating system to associate `.vtt` files with the CLI tool. They download a transcript and double-click the VTT file. The OS launches the tool with the file path as an argument, the tool converts the file to Markdown in the same directory, and the user finds the generated `.md` file ready to open.

**Independent Test**: After configuring file association (via OS-specific mechanisms), double-click a VTT file and verify that a Markdown file is generated in the same directory.

**Acceptance Scenarios**:
1. Given the tool is registered as the default handler for `.vtt` files on Windows, When user double-clicks a VTT file, Then the tool receives the file path (potentially quoted and with spaces) and generates the Markdown output
2. Given the tool is registered via a `.desktop` entry on Linux, When user double-clicks a VTT file in a file manager, Then the tool processes the file and generates output
3. Given the tool is invoked with a file path containing spaces or special characters, When the tool parses arguments, Then it correctly handles quoted paths and processes the file

### User Story P4 – Control output file behavior with flags

**Narrative**: A user wants flexibility in how output files are handled. They use `--force` to overwrite existing Markdown files without errors, `--stdout` to print output for piping to other tools, or `--no-clobber` to skip conversion when the output already exists. Each flag provides predictable, non-interactive behavior suitable for scripts and automation.

**Independent Test**: Run `vtt-to-md input.vtt --force` when `input.md` exists and verify that the existing file is overwritten without prompting.

**Acceptance Scenarios**:
1. Given an output Markdown file already exists, When the tool runs without flags, Then it exits with an error indicating the file exists and no overwrite occurred
2. Given an output Markdown file already exists, When the tool runs with `--force`, Then the existing file is overwritten with new content
3. Given the user runs the tool with `--stdout`, When conversion completes, Then the Markdown output is printed to stdout and no file is written
4. Given an output Markdown file already exists, When the tool runs with `--no-clobber`, Then conversion is skipped and the tool exits successfully with a message indicating the file was preserved

### User Story P5 – Receive clear error messages for common failures

**Narrative**: A user encounters an error (file not found, permission denied, corrupted VTT format). The tool prints a clear, actionable error message indicating what went wrong and exits with an appropriate exit code. The user can immediately understand the problem and take corrective action.

**Independent Test**: Run `vtt-to-md nonexistent.vtt` and verify the tool exits with code 66 (EX_NOINPUT) and prints a message indicating the file could not be found.

**Acceptance Scenarios**:
1. Given the input file does not exist, When the tool runs, Then it prints an error message indicating file not found and exits with code 66
2. Given the user lacks read permissions on the input file, When the tool runs, Then it prints an error message indicating permission denied and exits with code 77
3. Given the output directory lacks write permissions, When the tool runs, Then it prints an error message indicating permission denied and exits with code 77
4. Given the input file contains malformed VTT syntax that prevents parsing, When the tool runs, Then it prints an error message indicating parse failure and exits with code 65
5. Given invalid command-line arguments (e.g., missing required positional argument), When the tool runs, Then it prints usage help and exits with code 64

### Additional Stories (P3 - Optional Enhancements)

### User Story P6 – Customize unknown speaker label

**Narrative**: A user prefers a different label for unattributed segments (e.g., "Unidentified" instead of "Unknown"). They use the `--unknown-speaker` flag to specify a custom label, and the tool uses this label for all cues without speaker attribution.

**Independent Test**: Run `vtt-to-md input.vtt --unknown-speaker "Unidentified"` and verify unattributed cues appear under "Unidentified" in the output.

**Acceptance Scenarios**:
1. Given a VTT file with unattributed cues, When the tool runs with `--unknown-speaker "Narrator"`, Then the output uses "Narrator" as the speaker label for those cues

### User Story P7 – Optionally include timestamps

**Narrative**: A user wants to preserve timing information in the Markdown output for reference or synchronization purposes. They use a `--include-timestamps` flag to add timestamps at the beginning of each speaker turn or each utterance.

**Independent Test**: Run `vtt-to-md input.vtt --include-timestamps first` and verify that each speaker turn begins with a timestamp in the output Markdown.

**Acceptance Scenarios**:
1. Given a VTT file with timestamps, When the tool runs with `--include-timestamps first`, Then each speaker turn in the output begins with the timestamp of the first cue in that turn
2. Given a VTT file with timestamps, When the tool runs with `--include-timestamps each`, Then each consolidated utterance includes its original timestamp
3. Given the tool runs without the `--include-timestamps` flag, When conversion completes, Then no timestamps appear in the output Markdown

### Edge Cases

- Empty VTT file (no cues): Tool generates an empty or minimal Markdown file and exits successfully
- VTT file with only NOTE/STYLE/REGION blocks (no transcript cues): Tool generates empty transcript content or a note indicating no cues found
- Cue text containing HTML character references (e.g., `&amp;`, `&lt;`): Tool correctly decodes these to their literal characters
- Cue text with nested or malformed tags (e.g., unclosed `<i>`, mismatched `<v>`): Tool handles gracefully by closing spans at cue end or ignoring unknown tags
- VTT file with blank lines inside cue payloads (non-standard, seen in Google Meet exports): Tool strips or coalesces blank lines to produce valid consolidated text
- Speaker name is only whitespace after sanitization: Tool treats as unknown speaker
- Very large VTT file (multi-hour meeting, several MB): Tool processes efficiently using streaming/buffered reading without excessive memory usage
- File path argument contains spaces or special characters: Tool correctly handles quoted and unquoted paths as provided by the OS shell or file manager
- Output filename collision with input filename (e.g., user specifies `input.vtt` as output path): Tool detects this and exits with an error to prevent overwriting the source file

## Requirements

### Functional Requirements

- FR-001: Parse WebVTT files to extract cue text and speaker attributions from `<v>` voice tags (Stories: P1, P2)
- FR-002: Consolidate consecutive cues from the same speaker into single Markdown paragraphs (Stories: P1)
- FR-003: Generate Markdown output with bold speaker names followed by their consolidated text (Stories: P1)
- FR-004: Write output Markdown file to the same directory as input, using input filename with `.md` extension (Stories: P1)
- FR-005: Accept command-line argument specifying input VTT file path (Stories: P1, P3)
- FR-006: Accept optional command-line argument specifying output file path (Stories: P1)
- FR-007: Assign default speaker label (e.g., "Unknown") to cues lacking `<v>` tags or with empty annotations (Stories: P2)
- FR-008: Sanitize speaker names by removing @ symbols and escaping Markdown special characters (Stories: P1)
- FR-009: Provide `--force` flag to overwrite existing output files (Stories: P4)
- FR-010: Provide `--stdout` flag to print Markdown output to stdout instead of writing to file (Stories: P4)
- FR-011: Provide `--no-clobber` flag to skip conversion when output file exists (Stories: P4)
- FR-012: Detect and report file not found errors with exit code 66 (Stories: P5)
- FR-013: Detect and report permission errors with exit code 77 (Stories: P5)
- FR-014: Detect and report VTT parse errors with exit code 65 (Stories: P5)
- FR-015: Detect and report invalid CLI usage with exit code 64 (Stories: P5)
- FR-016: Exit with code 0 on successful conversion (Stories: P1)
- FR-017: Handle file paths containing spaces and special characters correctly (Stories: P3)
- FR-018: Decode HTML character references in speaker names and cue text (Stories: P1, P2)
- FR-019: Handle malformed or unclosed VTT tags gracefully without crashing (Stories: P2)
- FR-020: Strip or coalesce blank lines in cue payloads (non-standard but observed in real transcripts) (Stories: P2)
- FR-021: Trim and normalize whitespace in speaker names and cue text (Stories: P1, P2)
- FR-022: Provide `--unknown-speaker` flag to customize the default speaker label (Stories: P6)
- FR-023: Provide `--include-timestamps` flag with options `none`, `first`, `each` to control timestamp inclusion in output (Stories: P7)
- FR-024: Process large VTT files efficiently using streaming or buffered reading (Stories: P1)
- FR-025: Prevent overwriting input file when output path equals input path (Stories: P5)

### Key Entities

- **VTT File**: Input file in WebVTT format containing timestamped cues with optional speaker annotations and styling tags
- **Cue**: A single timestamped segment of text in the VTT file, potentially containing a `<v>` tag for speaker attribution
- **Speaker Segment**: One or more consecutive cues attributed to the same speaker, consolidated into a single Markdown paragraph
- **Markdown Output**: The generated file containing speaker segments formatted as `**SpeakerName:** text`

### Cross-Cutting / Non-Functional

- **Portability**: Tool must compile to a standalone executable for Windows, Linux, and macOS without runtime dependencies beyond OS shell (Rationale: enables file association and simplifies distribution)
- **Usability**: Error messages must be clear, actionable, and reference specific file paths or issues encountered
- **Robustness**: Tool must handle common VTT variations from Teams, Zoom, and Meet without requiring manual preprocessing
- **Performance**: Tool should process typical transcript files (up to several MB) in under a second on modern hardware

## Success Criteria

- SC-001: Tool successfully parses VTT files from Microsoft Teams, Zoom, and Google Meet containing `<v>` tags, extracting speaker and text (FR-001)
- SC-002: Consecutive cues with identical speaker names are consolidated into single Markdown paragraphs without duplicate speaker labels (FR-002)
- SC-003: Generated Markdown output matches format `**SpeakerName:** text` with properly escaped special characters (FR-003, FR-008)
- SC-004: Output file is created in the same directory as input file with `.md` extension (FR-004)
- SC-005: Tool exits with code 0 when conversion succeeds, 64 for usage errors, 65 for parse errors, 66 for missing files, 77 for permission errors (FR-012, FR-013, FR-014, FR-015, FR-016)
- SC-006: Tool handles file paths with spaces and special characters when invoked from command line or via file association (FR-005, FR-017)
- SC-007: Cues without `<v>` tags or with empty annotations appear under configurable default speaker label (default "Unknown") (FR-007, FR-022)
- SC-008: Tool overwrites existing output files only when `--force` flag is provided (FR-009)
- SC-009: Tool prints Markdown to stdout (no file written) when `--stdout` flag is provided (FR-010)
- SC-010: Tool skips conversion and exits successfully when output exists and `--no-clobber` is provided (FR-011)
- SC-011: Error messages for file not found, permission denied, and parse errors include relevant file paths and actionable guidance (FR-012, FR-013, FR-014)
- SC-012: Tool processes multi-MB VTT files without excessive memory usage or crashes (FR-024)
- SC-013: Tool prevents overwriting input file when output path is identical to input path (FR-025)
- SC-014: HTML character references in speaker names and cue text are correctly decoded (FR-018)
- SC-015: Malformed VTT tags do not cause crashes or produce corrupt output (FR-019)
- SC-016: When `--include-timestamps` is used, timestamps appear in the output as specified by the flag option (FR-023)

## Assumptions

- Default speaker label is "Unknown" unless customized via `--unknown-speaker` flag (Rationale: common neutral term, can be overridden if needed)
- Output file should not be created or overwritten by default when it already exists (user must use `--force`) (Rationale: prevents accidental data loss, aligns with CLI best practices per clig.dev)
- Whitespace consolidation joins adjacent same-speaker cues with a single space unless cue ends with terminal punctuation (Rationale: produces natural reading flow)
- Speaker name normalization includes Unicode NFC normalization (Rationale: prevents combining character issues)
- Exit codes follow BSD `sysexits.h` conventions for cross-platform consistency (Rationale: widely recognized, self-documenting)
- Tool does not open the generated Markdown file automatically unless a future `--open` flag is implemented (Rationale: keeps default behavior non-interactive and script-friendly)
- Tool does not parse or preserve STYLE, REGION, or NOTE blocks beyond skipping them (Rationale: these are not relevant to plain transcript conversion)
- Speaker changes are assumed to occur at cue boundaries; mid-cue speaker changes are not handled (Rationale: VTT format does not support mid-cue speaker changes in standard usage)
- Timestamps are omitted from output by default (Rationale: improves readability; users can opt-in via flag if needed)

## Scope

### In Scope

- Command-line tool accepting VTT file path and optional output path
- Parsing VTT format to extract cue text and speaker annotations
- Consolidating same-speaker cues and generating Markdown output
- Sanitizing speaker names (removing @, escaping Markdown special characters)
- File system operations: reading input, writing output, checking existence and permissions
- Flags: `--force`, `--no-clobber`, `--stdout`, `--unknown-speaker`, `--include-timestamps`
- Error handling with appropriate exit codes (64, 65, 66, 77, 0)
- Support for VTT variations from Teams, Zoom, and Meet
- Cross-platform compilation (Windows, Linux, macOS)

### Out of Scope

- Graphical user interface (GUI)
- Automatic opening of generated files (no `--open` flag in initial version)
- Advanced formatting options (e.g., italics, code blocks, custom Markdown templates)
- Multi-file batch processing in a single invocation
- Exporting to formats other than Markdown (JSON, HTML, plain text)
- Speaker statistics or analytics (word count, speaking time, turn-taking metrics)
- Integration with transcription services or APIs
- Preserving or converting VTT STYLE and REGION blocks
- Handling SRT or other subtitle formats (only WebVTT)
- Network operations (downloading files, fetching metadata)
- Configuration files or persistent settings
- Localization or internationalization of error messages
- Automatic file association registration (users must configure this manually per OS)

## Dependencies

- Rust toolchain (for building the executable)
- Standard library file I/O and path manipulation
- Command-line argument parsing library (`clap` or equivalent)
- Error handling library (`anyhow`, `thiserror`, or `miette`)
- Optional: Regular expressions library (`regex`) if used for parsing
- Optional: Unicode normalization library (`unicode-normalization`) for speaker name sanitization

## Risks & Mitigations

- **Risk**: VTT files from different platforms may use inconsistent or non-standard formatting (e.g., Google Meet's blank lines, missing `<v>` tags). **Impact**: Parsing failures or incorrect output. **Mitigation**: Implement permissive parsing that tolerates common deviations; strip blank lines, handle missing `<v>` tags with default labels, close unclosed tags at cue end.

- **Risk**: Speaker names may contain characters that break Markdown formatting (e.g., `*`, `_`, `[`, `]`). **Impact**: Unintended bold/italic/link formatting in output. **Mitigation**: Escape Markdown special characters in speaker names using backslash per CommonMark spec.

- **Risk**: Users may accidentally overwrite important files if output path is misconfigured. **Impact**: Data loss. **Mitigation**: Default behavior is to error when output exists; require explicit `--force` flag to overwrite. Detect and prevent overwriting input file.

- **Risk**: File association may pass paths with spaces or special characters that are not properly quoted or escaped. **Impact**: Tool fails to find input file. **Mitigation**: Use Rust's standard path handling which correctly processes OS-provided arguments; document proper file association configuration for each OS (quoting `%1` on Windows, using `%F` on Linux).

- **Risk**: Very large VTT files (multi-hour meetings) may cause memory exhaustion if entire file is loaded into memory. **Impact**: Tool crashes or system becomes unresponsive. **Mitigation**: Use buffered/streaming file reading (`BufRead` and `Lines` iterator) to process files incrementally.

- **Risk**: Exit codes may not be correctly mapped, causing confusion in scripts or automation. **Impact**: Incorrect error handling in calling scripts. **Mitigation**: Use explicit `std::process::ExitCode` returns and document exit code meanings; follow `sysexits.h` conventions consistently.

## References

- Issue: https://github.com/lossyrob/vtt-to-md/issues/1
- Research: .paw/work/vtt-to-md-cli/SpecResearch.md
- External:
  - WebVTT Specification (W3C): https://www.w3.org/TR/webvtt1/
  - BSD sysexits.h: https://man7.org/linux/man-pages/man3/sysexits.h.3head.html
  - Command Line Interface Guidelines: https://clig.dev/
  - CommonMark Specification: https://github.com/commonmark/commonmark-spec

## Glossary

- **WebVTT**: Web Video Text Tracks format, a W3C standard for timed text tracks (captions, subtitles, transcripts)
- **Voice tag (`<v>`)**: WebVTT cue text tag used to identify the speaker of a segment
- **Cue**: A single timestamped text segment in a WebVTT file
- **Annotation**: The text following a tag name in VTT syntax (e.g., speaker name after `<v`)
- **BSD sysexits**: Standard Unix exit codes (from `sysexits.h`) indicating specific error categories
- **CommonMark**: A strongly specified variant of Markdown with unambiguous parsing rules
- **File association**: OS-level configuration mapping file extensions to default applications or handlers
