# VTT to MD CLI Implementation Plan

## Overview

We're building a Rust command-line tool that converts WebVTT (Web Video Text Tracks) transcript files to readable Markdown format. The tool will parse VTT files downloaded from meeting platforms (Microsoft Teams, Zoom, Google Meet), extract speaker segments, consolidate consecutive utterances from the same speaker, and generate clean Markdown documents with bold speaker names followed by their text.

## Current State Analysis

The project is currently scaffolded with:
- `Cargo.toml`: Rust 2024 edition, version 0.1.0, no dependencies yet
- `src/main.rs`: Basic "Hello, world!" placeholder
- `README.md`: Basic build/run instructions

**Key Constraints:**
- Rust 2024 edition support available
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
2. **CLI Parsing**: Implement argument/flag parsing
3. **VTT Parser**: Build robust parser for WebVTT format with speaker extraction
4. **Consolidation**: Implement speaker-turn consolidation logic
5. **Markdown Generation**: Format and write output with proper escaping
6. **Testing**: Add comprehensive test coverage

Each phase includes both automated verification (cargo commands) and manual verification (testing with real VTT files).

## Architecture Overview

### Module Structure
```
src/
├── main.rs           # Entry point, CLI orchestration, error handling
├── cli.rs            # Argument parsing and validation
├── parser.rs         # VTT file parsing and cue extraction
├── consolidator.rs   # Speaker segment consolidation
├── markdown.rs       # Markdown generation and file output
└── error.rs          # Error types and exit code mapping
```

### Data Flow
```
VTT File → Parser → Cues → Consolidator → Segments → Markdown → Output File/Stdout
                ↓                                        ↓
            Validation                              Error Handling
```

### Key Design Decisions

**Error Handling Strategy:**
- Use `thiserror` for custom error types with clear messages
- Use `anyhow` for error context in application logic
- Map errors to BSD sysexits.h conventions (EX_NOINPUT=66, EX_NOPERM=77, EX_DATAERR=65, EX_IOERR=74, EX_CANTCREAT=73, EX_USAGE=64)
- Never panic on expected errors; return appropriate exit codes

**Parsing Strategy:**
- Use regex for pattern matching (`<v>` tags, timestamps)
- Be permissive: skip unknown blocks (NOTE, STYLE, REGION), handle missing tags
- Validate WEBVTT header exists at file start
- Strip/normalize whitespace and decode HTML entities (&amp;, &lt;, etc.)

**Speaker Handling:**
- Extract from `<v Speaker>text</v>` voice tags
- Sanitize: remove @ symbols, apply Unicode NFC normalization, escape Markdown special chars
- Default to configurable label (e.g., "Unknown") when speaker missing
- Consolidate consecutive same-speaker cues with intelligent whitespace joining

**CLI Design:**
- Positional: input file (required), output file (optional, defaults to input.md)
- Flags: --force (overwrite), --no-clobber (skip if exists), --stdout (print instead of write)
- Options: --unknown-speaker (custom label), --include-timestamps (none/first/each)
- Default behavior: safe (error if output exists, require explicit --force)

**Output Format:**
- Markdown paragraphs: `**SpeakerName:** consolidated text`
- Double newline between speaker turns
- Optional timestamps: `[HH:MM:SS.mmm] **Speaker:** text`

---

## Phase 1: Project Setup and Dependencies

### Objective
Establish the foundational project structure with appropriate dependencies and error handling infrastructure that will support all subsequent phases.

### What Needs to Be Done

#### Dependencies to Add
Update `Cargo.toml` with:
- **clap** (v4.5+): CLI argument parsing with derive macros, colored help text, and suggestions
- **thiserror** (v1.0+): Define custom error types with Display implementations
- **anyhow** (v1.0+): Error context chaining for application-level error handling
- **regex** (v1.10+): Pattern matching for VTT voice tags and timestamps
- **unicode-normalization** (v0.1+): NFC normalization for speaker names

**Rationale**: These dependencies align with Rust ecosystem best practices and provide the necessary tools without excessive complexity. Clap's derive feature offers ergonomic CLI definition. The thiserror/anyhow combination separates library errors (thiserror) from application errors (anyhow).

#### Error Type System
Create `src/error.rs` defining a `VttError` enum with variants for:
- File I/O errors (read, write, not found, permission denied)
- Parse errors (invalid VTT format, missing header, malformed content)
- Application errors (output file exists, input/output same file)

Each error variant must:
- Include relevant context (file paths, reasons)
- Map to appropriate BSD sysexits.h exit codes
- Provide clear, actionable error messages

**Exit Code Mapping:**
- 0: Success
- 64 (EX_USAGE): CLI usage errors
- 65 (EX_DATAERR): VTT parse errors, invalid data
- 66 (EX_NOINPUT): Input file not found
- 73 (EX_CANTCREAT): Output file already exists (without --force)
- 74 (EX_IOERR): File I/O errors
- 77 (EX_NOPERM): Permission denied

#### Module Scaffolding
Create empty module files with documentation comments describing their purpose:
- `src/cli.rs`: Will handle command-line argument parsing
- `src/parser.rs`: Will parse VTT files and extract cues
- `src/consolidator.rs`: Will consolidate speaker segments
- `src/markdown.rs`: Will generate Markdown output

Update `src/main.rs` to declare these modules and establish the basic entry point structure with proper error handling (returning ExitCode).

### Success Criteria

#### Automated Verification
- [x] Project compiles cleanly: `cargo build`
- [x] All dependencies resolve: `cargo check`
- [x] No clippy warnings: `cargo clippy -- -D warnings`
- [x] Code formatting is correct: `cargo fmt --check`
- [x] Error types compile and can be instantiated

#### Manual Verification
- [x] Review `Cargo.toml` dependencies match requirements
- [x] Verify error module defines all required variants per Spec.md
- [x] Confirm exit codes follow sysexits.h conventions
- [x] Check module structure is clean and documented
- [x] Ensure error messages are user-friendly and actionable

#### Critical Questions to Answer
- Are there any dependency conflicts or version issues? **No - all dependencies resolved cleanly**
- Do error messages provide enough context for users to take action? **Yes - each error includes relevant paths and actionable guidance**
- Is the module structure intuitive and maintainable? **Yes - clear separation of concerns with well-documented module purposes**

### Phase 1 Implementation Complete

**Status**: ✅ Complete

**Summary**: Successfully established project foundation with all required dependencies and error handling infrastructure. Created modular structure with clear separation of concerns.

**Key Accomplishments**:
- Added all required dependencies (clap 4.5, thiserror 1.0, anyhow 1.0, regex 1.10, unicode-normalization 0.1) with appropriate features
- Implemented comprehensive VttError enum with 8 error variants covering all failure modes
- Mapped errors to BSD sysexits.h exit codes (64, 65, 66, 73, 74, 77)
- Created module scaffolding (cli, parser, consolidator, markdown) with documentation
- Updated main.rs with ExitCode return type and module declarations
- All automated verification passes (build, check, clippy, fmt)

**Notes for Future Phases**:
- Error types are ready for use in subsequent phases
- Module structure provides clear locations for implementing each phase's functionality
- The `#[allow(dead_code)]` annotation on error.rs will be removed once errors are used in Phase 2+

---

## Phase 2: CLI Argument Parsing

### Objective
Implement a user-friendly command-line interface that accepts file paths, validates arguments, and provides clear help documentation and error messages.

### What Needs to Be Done

#### Argument Structure
Define CLI using clap's derive macros in `src/cli.rs`:

**Positional Arguments:**
- `input`: Required path to VTT file
- `output`: Optional path to output MD file (defaults to input filename with .md extension)

**Flags:**
- `--force` / `-f`: Overwrite existing output file
- `--no-clobber` / `-n`: Skip conversion if output exists
- `--stdout`: Print Markdown to stdout instead of writing file

**Options:**
- `--unknown-speaker <LABEL>`: Custom label for cues without speaker attribution (default: "Unknown")
- `--include-timestamps <MODE>`: Timestamp inclusion mode (values: none, first, each; default: none)

**Built-in:**
- `--help` / `-h`: Display help text (automatically provided by clap)
- `--version` / `-V`: Display version (automatically provided by clap)

#### Validation Logic
Implement validation that:
- Checks for conflicting flags (--force and --no-clobber cannot both be used)
- Detects if output path equals input path (prevent overwriting source)
- Handles paths with spaces and special characters correctly
- Provides method to derive output path from input path when not specified

#### Integration with Main
Update `src/main.rs` to:
- Parse arguments and catch clap errors (map to exit code 64)
- Run validation before processing
- Display errors to stderr with appropriate exit codes

### Success Criteria

#### Automated Verification
- [x] Project compiles: `cargo build`
- [x] Help text displays correctly: `cargo run -- --help`
- [x] Version flag works: `cargo run -- --version`
- [x] Invalid arguments trigger usage error: `cargo run -- --invalid` (exit 64)
- [x] Conflicting flags detected: `cargo run -- input.vtt --force --no-clobber` (error exit)
- [x] Missing required argument shows error: `cargo run --` (exit 64)

#### Manual Verification
- [x] Run with file path and verify output path derivation is correct
- [x] Test path with spaces: `cargo run -- "my file.vtt"`
- [x] Test custom output: `cargo run -- input.vtt custom.md`
- [x] Test all flags parse correctly: `cargo run -- input.vtt --force --stdout`
- [x] Test unknown speaker option: `cargo run -- input.vtt --unknown-speaker "Narrator"`
- [x] Test timestamp modes: `cargo run -- input.vtt --include-timestamps first`
- [x] Verify help text is clear, well-formatted, and describes all options
- [x] Verify error messages for invalid input are actionable

#### Critical Questions to Answer
- Does the CLI feel intuitive to use? **Yes - argument names are clear and help text is comprehensive**
- Are error messages clear about what went wrong and how to fix it? **Yes - clap provides detailed error messages with suggestions**
- Do paths with spaces work on all target platforms (Windows, Linux, macOS)? **Yes - tested on Windows, paths with spaces work correctly**

### Phase 2 Implementation Complete

**Status**: ✅ Complete

**Summary**: Successfully implemented CLI argument parsing with comprehensive validation and user-friendly help text.

**Key Accomplishments**:
- Implemented Args struct with clap derive macros supporting all required arguments, flags, and options
- Added validation logic for conflicting flags (--force vs --no-clobber) and same-file detection
- Implemented automatic output path derivation from input path (replaces .vtt with .md)
- Integrated CLI parsing with main.rs, including proper error handling for clap errors
- All automated verification passes: build, help, version, invalid args, conflicting flags, missing args
- All manual verification passes: paths with spaces, custom output, all flags, unknown speaker option, timestamp modes
- Help text is clear, comprehensive, and well-formatted
- Clap provides helpful error messages with suggestions for invalid input

**Notes for Future Phases**:
- CLI module is ready for use in conversion logic
- TimestampMode enum will be used by consolidator and markdown modules
- Args validation ensures safe operation (prevents overwriting input file)
- All flags and options are properly parsed and available for use

---

## Phase 3: VTT Parser Implementation

### Objective
Build a robust parser that reads VTT files, extracts speaker attributions and cue text, and handles real-world variations from different meeting platforms without failing.

### What Needs to Be Done

#### Data Structures
Define types in `src/parser.rs`:
- `Cue`: Represents a single VTT cue with optional timestamp, optional speaker, and text content
- `VttDocument`: Contains a collection of parsed cues

#### File Reading and Validation
Implement file reading that:
- Opens and reads VTT file, handling file-not-found and permission errors appropriately
- Validates that file starts with "WEBVTT" header
- Returns descriptive errors with file paths for debugging

#### Cue Extraction Logic
Parse VTT content by:
- Identifying timestamp lines (format: `HH:MM:SS.mmm --> HH:MM:SS.mmm`)
- Extracting cue text (lines following timestamp until empty line or next timestamp)
- Skipping metadata blocks (NOTE, STYLE, REGION) without failing
- Handling blank lines in cue payloads (non-standard but seen in Google Meet exports)

#### Speaker Extraction
Extract speaker names from voice tags:
- Use regex to match `<v SpeakerName>text</v>` patterns
- Handle missing voice tags (treat as unknown speaker)
- Handle empty voice annotations (e.g., `<v>`) gracefully

#### Text Cleaning
Implement text sanitization:
- Strip HTML tags other than voice tags already processed
- Decode HTML character references (&amp; → &, &lt; → <, &gt; → >, &quot; → ", &#39; → ')
- Normalize whitespace (collapse multiple spaces, trim)
- Remove or coalesce blank lines within cue text

#### Speaker Name Sanitization
Sanitize speaker names by:
- Removing @ symbols (used for anonymized users in Teams)
- Applying Unicode NFC normalization to prevent combining character issues
- Escaping Markdown special characters (* _ # [ ] ( ) { } ! > | ` \) with backslashes
- Trimming whitespace
- Treating whitespace-only names as unknown speakers

#### Platform-Specific Handling
Ensure parser handles known variations:
- **Microsoft Teams**: Standard `<v>` tags with potential @ symbols
- **Zoom**: May use numeric speaker placeholders instead of names
- **Google Meet**: Often missing `<v>` tags, may have blank lines in cue payloads

### Success Criteria

#### Automated Verification
- [ ] Code compiles: `cargo build`
- [ ] Unit test: Parse valid VTT file with multiple speakers succeeds
- [ ] Unit test: File without WEBVTT header returns parse error
- [ ] Unit test: Missing input file returns file-not-found error (exit 66)
- [ ] Unit test: Speaker sanitization removes @ and escapes Markdown chars
- [ ] Unit test: HTML entities are decoded correctly
- [ ] Unit test: Cues without `<v>` tags are assigned None for speaker
- [ ] Unit test: Empty or whitespace-only speaker names treated as None

#### Manual Verification
- [ ] Parse a real Microsoft Teams VTT file successfully
- [ ] Verify speaker names extracted from `<v>` tags
- [ ] Verify @ symbols removed from anonymized speakers
- [ ] Parse a Zoom transcript (if available) without crashing
- [ ] Parse a Google Meet transcript with blank lines without failing
- [ ] Test file with malformed/unclosed tags—parser should not crash
- [ ] Test file with only NOTE/STYLE blocks—should return no-cues error
- [ ] Verify HTML entities in text (&amp;, &lt;) are decoded
- [ ] Verify Unicode characters in speaker names are handled correctly

#### Critical Questions to Answer
- Does the parser handle all platform variations without requiring manual preprocessing?
- Are error messages specific enough to help users understand what went wrong?
- Does speaker sanitization preserve readability while preventing Markdown formatting issues?

---

## Phase 4: Speaker Consolidation

### Objective
Implement logic to merge consecutive cues from the same speaker into coherent paragraphs while maintaining natural reading flow and respecting sentence boundaries.

### What Needs to Be Done

#### Data Structure
Define `SpeakerSegment` type in `src/consolidator.rs` containing:
- Speaker name (string)
- Consolidated text (string)
- Optional timestamp (for first cue of segment)

#### Consolidation Algorithm
Implement consolidation that:
- Iterates through parsed cues in order
- Detects speaker changes (comparing current speaker with previous)
- When speaker changes: saves previous segment, starts new segment
- When speaker continues: appends current cue text to ongoing segment
- Handles unknown speakers by applying configurable label (from CLI argument)

#### Text Joining Strategy
Join text intelligently:
- Add single space between cues from same speaker
- Respect sentence boundaries (don't incorrectly merge sentences)
- Handle cases where cue ends with terminal punctuation (. ? !)
- Trim final consolidated text to remove leading/trailing whitespace

#### Timestamp Handling
Support timestamp modes (based on CLI flag):
- **None**: Don't include timestamps in segments
- **First**: Include timestamp from first cue of each speaker turn
- **Each**: Include timestamp for each original cue (requires different data structure or annotation)

#### Edge Cases
Handle:
- Empty VTT files (no cues)
- Files with only one speaker
- Alternating speakers (A, B, A, B pattern)
- Cues with empty/whitespace-only text (skip them)
- Very long speaker turns (ensure memory efficiency)

### Success Criteria

#### Automated Verification
- [ ] Code compiles: `cargo build`
- [ ] Unit test: Consecutive same-speaker cues consolidate into single segment
- [ ] Unit test: Different speakers create separate segments in order
- [ ] Unit test: Unknown speakers get custom label applied
- [ ] Unit test: Empty/whitespace cues are skipped
- [ ] Unit test: Text joining preserves spacing correctly
- [ ] Unit test: Timestamp modes work (none, first, each)

#### Manual Verification
- [ ] Create VTT with 3 consecutive cues from same speaker; verify single segment in output
- [ ] Create VTT with alternating speakers (A, B, A, B); verify 4 segments in correct order
- [ ] Test with file having no speaker tags; verify "Unknown" label applied
- [ ] Test with custom unknown speaker label; verify label used correctly
- [ ] Inspect joined text for natural reading flow (no missing spaces, no double spaces)
- [ ] Verify sentence boundaries are respected (no run-on sentences)

#### Critical Questions to Answer
- Does the consolidation produce readable, natural-sounding paragraphs?
- Are speaker changes clearly delineated?
- Does the whitespace handling work correctly across different input formats?

---

## Phase 5: Markdown Generation and File Output

### Objective
Generate properly formatted Markdown output and handle file writing with appropriate safeguards, supporting multiple output modes (file, stdout) and overwrite behaviors.

### What Needs to Be Done

#### Markdown Formatting
Implement in `src/markdown.rs`:
- Format each speaker segment as: `**SpeakerName:** text\n\n`
- Speaker names are bold using `**...**` syntax
- Double newline between segments for paragraph separation
- If timestamps enabled, prepend: `[HH:MM:SS.mmm] **SpeakerName:** text\n\n`

#### File Writing Logic
Implement output handling:
- Check if output file already exists
- If exists and --force not set: return error (exit 73, EX_CANTCREAT)
- If exists and --force set: overwrite file
- Handle write permission errors (exit 77, EX_NOPERM)
- Handle other I/O errors (exit 74, EX_IOERR)

#### Stdout Mode
When --stdout flag is set:
- Skip file writing logic entirely
- Print formatted Markdown to stdout
- Handle stdout write errors gracefully

#### No-Clobber Mode
When --no-clobber flag is set and output exists:
- Skip conversion silently (or with informational message)
- Exit successfully (code 0)
- Do not treat as error

#### Integration with Main
Wire up all components in `src/main.rs`:
- Parse CLI arguments
- Read and parse VTT file
- Consolidate speaker segments
- Generate Markdown
- Write output according to selected mode (file/stdout)
- Handle errors at each stage with appropriate exit codes

### Success Criteria

#### Automated Verification
- [ ] Full build succeeds: `cargo build --release`
- [ ] Basic conversion: Create test.vtt, run tool, verify test.md created
- [ ] Force flag: Run twice with --force, second run succeeds
- [ ] No-clobber flag: Run twice with --no-clobber, second run skips with message
- [ ] Stdout flag: `cargo run -- test.vtt --stdout` prints Markdown to console
- [ ] Clippy passes: `cargo clippy -- -D warnings`
- [ ] Formatting passes: `cargo fmt --check`

#### Manual Verification
- [ ] Convert real Teams VTT file; inspect Markdown for quality
- [ ] Verify speaker names are bold and properly formatted (`**Name:**`)
- [ ] Verify paragraphs are separated by double newlines
- [ ] Verify consecutive same-speaker text is consolidated
- [ ] Test with Zoom VTT file; verify compatibility
- [ ] Test with Google Meet VTT file; verify compatibility
- [ ] Test error case: nonexistent file (verify exit 66 and clear message)
- [ ] Test error case: write-protected directory (verify exit 77)
- [ ] Test output file placed in same directory as input
- [ ] Test file path with spaces works on Windows
- [ ] Verify Markdown special chars in speaker names are escaped
- [ ] Test timestamp modes (none, first, each) and verify correct output format

#### Critical Questions to Answer
- Is the Markdown output properly formatted and readable in standard Markdown viewers?
- Are all error cases handled with clear, actionable messages?
- Does the tool behave safely by default (no accidental overwrites)?

---

## Phase 6: Testing Strategy

### Objective
Establish comprehensive test coverage across unit, integration, and manual testing to ensure reliability and correctness across different platforms and edge cases.

### What Needs to Be Done

#### Unit Tests
Add test modules to each source file testing isolated functionality:

**Parser Tests (`src/parser.rs`):**
- Valid VTT file parsing succeeds
- Missing WEBVTT header returns error
- Speaker extraction from `<v>` tags works
- Speaker sanitization (@ removal, Markdown escaping) works
- HTML entity decoding works
- Cues without speakers handled correctly
- Malformed tags don't crash parser
- Empty files handled gracefully

**Consolidator Tests (`src/consolidator.rs`):**
- Same-speaker consecutive cues consolidate
- Different speakers create separate segments
- Unknown speaker label applied correctly
- Empty/whitespace cues skipped
- Text joining produces correct spacing
- Timestamp modes work as expected

**Markdown Tests (`src/markdown.rs`):**
- Segment formatting produces correct Markdown syntax
- Speaker names are bold
- Timestamps included when requested
- Double newlines between paragraphs

**Error Tests (`src/error.rs`):**
- Exit codes map correctly
- Error messages include relevant context

#### Integration Tests
Create `tests/integration_test.rs` with end-to-end scenarios:
- Basic conversion (input.vtt → input.md) succeeds
- Output file contains expected formatted content
- --force flag allows overwrite
- --no-clobber flag skips when output exists
- --stdout flag prints to console instead of file
- --unknown-speaker flag applies custom label
- --include-timestamps flags affect output format
- File-not-found error returns exit code 66
- Invalid VTT format returns exit code 65
- Permission errors return exit code 77

#### Manual Test Documentation
Create `tests/MANUAL_TESTS.md` checklist documenting:
- Platform-specific tests (Teams, Zoom, Meet VTT files)
- File association testing (Windows, Linux, macOS)
- Large file performance testing (multi-hour transcripts)
- Special character handling (spaces in paths, Unicode in names, emoji)
- Error condition testing (missing files, permissions, invalid format)
- Cross-platform compatibility verification

#### Test Dependencies
Add to `Cargo.toml` dev-dependencies:
- **tempfile** (v3.8+): For creating temporary test files and directories

### Success Criteria

#### Automated Verification
- [ ] All unit tests pass: `cargo test`
- [ ] All integration tests pass: `cargo test --test integration_test`
- [ ] Test coverage > 80% for core logic (parser, consolidator, markdown)
- [ ] Tests run successfully on CI for Windows, Linux, macOS (if CI configured)
- [ ] No test flakiness or race conditions

#### Manual Verification
- [ ] Complete all test cases in MANUAL_TESTS.md
- [ ] Test with real VTT files from Teams, Zoom, and Meet
- [ ] Verify file association works on at least one platform
- [ ] Test large files (multi-hour transcripts) for performance
- [ ] Test paths with spaces on Windows
- [ ] Test Unicode speaker names and emoji
- [ ] Test all error conditions produce appropriate exit codes and messages
- [ ] Cross-platform testing confirms compatibility

#### Critical Questions to Answer
- Are there any gaps in test coverage for critical functionality?
- Do tests accurately reflect real-world usage scenarios?
- Are error messages tested to ensure they're helpful?
- Is the manual testing process repeatable and documented?

---

## Testing Strategy Summary

### Unit Testing Focus
- Individual functions and methods in isolation
- Edge cases and error conditions
- Data transformation correctness
- Speaker name sanitization
- HTML entity decoding
- Whitespace normalization

### Integration Testing Focus
- End-to-end conversion workflows
- CLI argument handling and validation
- Flag behavior (--force, --no-clobber, --stdout)
- Error propagation and exit codes
- File I/O operations

### Manual Testing Focus
- Real-world VTT file compatibility (Teams, Zoom, Meet)
- Platform-specific behaviors (Windows, Linux, macOS)
- File association workflows
- Performance with large files
- Unicode and special character handling
- User experience and error message quality

### Test Data Requirements
Prepare minimal test VTT files for:
- Two-speaker conversation
- Single speaker multiple cues
- Missing voice tags
- Malformed/unclosed tags
- HTML entities in text
- Empty file
- No WEBVTT header
- Platform-specific variations (Teams @users, Google Meet blank lines)

---

## Performance Considerations

### Expected File Sizes
Typical meeting transcripts:
- 1-hour meeting: ~80-400 KB
- Multi-hour meeting: few MB maximum

### Performance Requirements
- Parse and convert typical file (< 1 MB) in under 1 second
- Handle multi-hour transcripts (few MB) in under 5 seconds
- Memory usage should be reasonable (< 100 MB for typical files)

### Implementation Strategy
- Use buffered file reading (`BufRead`) for efficient I/O
- Avoid loading entire file into memory if not necessary
- Regex compilation should be done once, not per-cue
- String allocations should be minimized where possible

### Optimization Approach
- Profile before optimizing
- Focus on correctness first, performance second
- Only optimize if real-world usage shows performance issues

---

## Migration and Deployment

### Migration Notes
N/A - This is a new tool with no existing users or data to migrate.

### Deployment Considerations
- Compile release binary: `cargo build --release`
- Binary location: `target/release/vtt-to-md.exe` (Windows) or `target/release/vtt-to-md` (Unix)
- No runtime dependencies beyond OS standard libraries
- Cross-platform binaries should be built on respective platforms or via cross-compilation

### File Association Setup (User Documentation)
**Windows:**
- Right-click .vtt file → Properties → Change → Browse to vtt-to-md.exe
- Or: Use `ftype` and `assoc` commands to register file type

**Linux:**
- Create `.desktop` file with `Exec=vtt-to-md %F`
- Place in `~/.local/share/applications/`
- Update MIME database if needed

**macOS:**
- File associations typically handled at application level, not for CLI tools
- Users can create Automator workflows or scripts if desired

---

## Open Questions and Risks

### Technical Risks

**Risk:** VTT files from different platforms may have unexpected format variations not covered by research
- **Mitigation:** Be permissive in parsing, log/skip unknown structures, test with real files from all platforms

**Risk:** Regex patterns may not match all VTT voice tag variations
- **Mitigation:** Make regex flexible, handle missing matches gracefully, provide clear errors when parsing fails

**Risk:** Large files could cause memory issues if entire file loaded at once
- **Mitigation:** Use streaming/buffered reading, process line-by-line or in chunks

### User Experience Risks

**Risk:** Error messages may not be clear enough for non-technical users
- **Mitigation:** Test error messages with non-developers, include file paths and actionable suggestions

**Risk:** File association may not work as expected across all platforms
- **Mitigation:** Provide clear documentation, test on each platform, consider alternative invocation methods

### Scope Risks

**Risk:** Feature creep (requests for JSON export, analytics, GUI, etc.)
- **Mitigation:** Clearly document out-of-scope features, focus on core conversion functionality first

---

## Success Metrics

### Functional Success
- Tool correctly converts VTT files from Teams, Zoom, and Meet without manual preprocessing
- Generated Markdown is readable and properly formatted
- All exit codes are appropriate for error conditions
- All CLI flags work as documented

### Quality Success
- Code passes clippy with no warnings
- Code is formatted per `cargo fmt`
- Test coverage > 80% for core logic
- No panics on expected inputs (including malformed VTT files)

### User Success
- Users can convert transcripts with a single command
- Error messages are clear and actionable
- Help text is sufficient for users to understand options
- Tool works across Windows, Linux, and macOS

---

## References

### Project Documentation
- Original Issue: https://github.com/lossyrob/vtt-to-md/issues/1
- Feature Specification: `.paw/work/vtt-to-md-cli/Spec.md`
- Research Findings: `.paw/work/vtt-to-md-cli/SpecResearch.md`

### External Standards and Guidelines
- **WebVTT Specification (W3C):** https://www.w3.org/TR/webvtt1/
- **BSD sysexits.h:** https://man7.org/linux/man-pages/man3/sysexits.h.3head.html
- **Command Line Interface Guidelines:** https://clig.dev/
- **CommonMark Specification:** https://github.com/commonmark/commonmark-spec

### Rust Ecosystem
- **clap Documentation:** https://docs.rs/clap/
- **thiserror Documentation:** https://docs.rs/thiserror/
- **anyhow Documentation:** https://docs.rs/anyhow/
- **Rust CLI Book:** https://rust-cli.github.io/book/
