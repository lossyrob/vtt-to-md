# Manual Testing Checklist

This document outlines manual testing procedures for the vtt-to-md CLI tool to ensure comprehensive quality assurance across platforms and real-world scenarios.

## Platform-Specific Tests

### Microsoft Teams VTT Files

- [ ] Download a real Teams meeting transcript (.vtt file)
- [ ] Run: `vtt-to-md meeting.vtt`
- [ ] Verify output file `meeting.md` is created
- [ ] Inspect Markdown:
  - [ ] Speaker names are properly formatted with bold (`**Name:**`)
  - [ ] @ symbols in anonymized speakers are removed
  - [ ] Text is readable and properly consolidated
  - [ ] Paragraph breaks are appropriate (double newlines)
- [ ] Test with multi-hour transcript (> 1 hour)
- [ ] Verify conversion completes in reasonable time (< 5 seconds)

### Zoom VTT Files

- [ ] Download a Zoom transcript (.vtt file)
- [ ] Run: `vtt-to-md zoom_meeting.vtt`
- [ ] Verify output is created and readable
- [ ] Check for any Zoom-specific formatting issues
- [ ] Verify numeric speaker placeholders are handled correctly

### Google Meet VTT Files

- [ ] Download a Google Meet transcript (.vtt file)
- [ ] Run: `vtt-to-md meet_transcript.vtt`
- [ ] Verify output is created and readable
- [ ] Verify blank lines in cue payloads don't break parsing
- [ ] Check handling of missing voice tags

## Cross-Platform Compatibility

### Windows

- [ ] Build release binary: `cargo build --release`
- [ ] Test from PowerShell: `.\target\release\vtt-to-md.exe meeting.vtt`
- [ ] Test from Command Prompt: `target\release\vtt-to-md.exe meeting.vtt`
- [ ] Test path with spaces: `vtt-to-md.exe "C:\Users\Test User\meeting.vtt"`
- [ ] Test long paths (> 260 characters) if applicable
- [ ] Verify exit codes in PowerShell: `$LASTEXITCODE`
- [ ] Test file association:
  - [ ] Right-click .vtt file → Properties → Change → Browse to vtt-to-md.exe
  - [ ] Double-click .vtt file to convert
  - [ ] Verify .md file created in same directory

### Linux

- [ ] Build release binary: `cargo build --release`
- [ ] Test from bash: `./target/release/vtt-to-md meeting.vtt`
- [ ] Test path with spaces: `./vtt-to-md "path with spaces/meeting.vtt"`
- [ ] Test symlinks: Create symlink to vtt-to-md, verify it works
- [ ] Verify exit codes: `echo $?`
- [ ] Test file permissions:
  - [ ] Create write-protected directory
  - [ ] Attempt conversion to that directory
  - [ ] Verify exit code 77 (EX_NOPERM) and clear error message
- [ ] Test file association (optional):
  - [ ] Create .desktop file with `Exec=vtt-to-md %F`
  - [ ] Place in `~/.local/share/applications/`
  - [ ] Test double-click behavior

### macOS

- [ ] Build release binary: `cargo build --release`
- [ ] Test from terminal: `./target/release/vtt-to-md meeting.vtt`
- [ ] Test path with spaces: `./vtt-to-md "path with spaces/meeting.vtt"`
- [ ] Verify exit codes: `echo $?`
- [ ] Test on both Intel and Apple Silicon (if available)

## Special Character Handling

### Paths with Spaces

- [ ] Windows: `vtt-to-md "C:\Users\Test User\Documents\my meeting.vtt"`
- [ ] Linux/macOS: `vtt-to-md "path with spaces/my meeting.vtt"`
- [ ] Verify both input and output paths handle spaces correctly

### Unicode Speaker Names

- [ ] Create VTT with Unicode characters: `<v José García>text</v>`
- [ ] Run conversion and verify speaker name appears correctly in Markdown
- [ ] Test with emoji: `<v Alice 👋>text</v>` (should be escaped/handled)
- [ ] Test with combining characters and verify NFC normalization works
- [ ] Test with RTL languages (Arabic, Hebrew) if applicable

### Markdown Special Characters in Speaker Names

- [ ] Test speaker name with asterisks: `<v User*123>text</v>`
- [ ] Test speaker name with underscores: `<v User_Name>text</v>`
- [ ] Test speaker name with brackets: `<v [Admin]>text</v>`
- [ ] Verify all special characters are properly escaped in output

## Error Conditions

### File Not Found

- [ ] Run: `vtt-to-md nonexistent.vtt`
- [ ] Verify exit code 66 (EX_NOINPUT)
- [ ] Verify clear error message: "File not found: nonexistent.vtt" or similar

### Invalid VTT Format

- [ ] Create file without WEBVTT header
- [ ] Run conversion
- [ ] Verify exit code 65 (EX_DATAERR)
- [ ] Verify clear error message about missing WEBVTT header

### Output File Exists (Default Behavior)

- [ ] Create test.vtt and convert it
- [ ] Run conversion again without --force
- [ ] Verify exit code 73 (EX_CANTCREAT)
- [ ] Verify clear error message: "Output file exists, use --force to overwrite"
- [ ] Verify original output file is not modified

### Write Permission Denied

- [ ] Create read-only output directory
- [ ] Attempt conversion to that directory
- [ ] Verify exit code 77 (EX_NOPERM)
- [ ] Verify clear error message about permission denied

### Same Input and Output File

- [ ] Run: `vtt-to-md test.vtt test.vtt`
- [ ] Verify error and appropriate exit code
- [ ] Verify error message explains the problem

## CLI Flag Testing

### --force Flag

- [ ] Create test.vtt and convert it
- [ ] Modify test.md manually
- [ ] Run: `vtt-to-md test.vtt --force`
- [ ] Verify test.md is overwritten with new content

### --no-clobber Flag

- [ ] Create test.vtt and convert it
- [ ] Modify test.md manually
- [ ] Run: `vtt-to-md test.vtt --no-clobber`
- [ ] Verify exit code 0 (success)
- [ ] Verify test.md is NOT modified (retains manual changes)
- [ ] Verify informational message (if any)

### --stdout Flag

- [ ] Run: `vtt-to-md test.vtt --stdout`
- [ ] Verify Markdown is printed to console
- [ ] Verify no output file is created
- [ ] Test piping: `vtt-to-md test.vtt --stdout > output.md`
- [ ] Verify piped file contains correct Markdown

### --unknown-speaker Flag

- [ ] Create VTT without speaker tags
- [ ] Run: `vtt-to-md test.vtt --unknown-speaker "Narrator" --stdout`
- [ ] Verify output uses "Narrator" instead of "Unknown"

### --include-timestamps none (default)

- [ ] Run: `vtt-to-md test.vtt --stdout`
- [ ] Verify no timestamps appear in output

### --include-timestamps first

- [ ] Run: `vtt-to-md test.vtt --include-timestamps first --stdout`
- [ ] Verify timestamp appears before first cue of each speaker turn
- [ ] Format: `[HH:MM:SS.mmm] **Speaker:** text`

### --include-timestamps each

- [ ] Run: `vtt-to-md test.vtt --include-timestamps each --stdout`
- [ ] Verify timestamp appears for each original cue
- [ ] Multiple timestamps for same speaker if consecutive cues

### --help Flag

- [ ] Run: `vtt-to-md --help`
- [ ] Verify help text is clear and complete
- [ ] Verify all flags and options are documented
- [ ] Verify examples are provided (if any)

### --version Flag

- [ ] Run: `vtt-to-md --version`
- [ ] Verify version number displays correctly (should match Cargo.toml)

## Performance Testing

### Large Files

- [ ] Find or create multi-hour transcript (several MB)
- [ ] Run conversion and time it: `time vtt-to-md large_meeting.vtt`
- [ ] Verify completion in under 5 seconds
- [ ] Monitor memory usage (should stay under 100 MB)

### Very Long Speaker Turns

- [ ] Create VTT with 100+ consecutive cues from same speaker
- [ ] Run conversion
- [ ] Verify text is properly consolidated into single paragraph
- [ ] Verify no memory issues or slowdowns

## User Experience

### Error Message Quality

For each error condition tested above:
- [ ] Verify error message is written to stderr (not stdout)
- [ ] Verify message clearly states what went wrong
- [ ] Verify message suggests how to fix the problem (if applicable)
- [ ] Verify file paths are included in error messages
- [ ] Verify no stack traces or debug output in production build

### Output Quality

- [ ] Generated Markdown is readable in:
  - [ ] Visual Studio Code preview
  - [ ] GitHub markdown viewer
  - [ ] Obsidian or similar markdown editor
  - [ ] Plain text editor
- [ ] Speaker names stand out (bold formatting works)
- [ ] Paragraph breaks make conversation flow clear
- [ ] No unwanted artifacts (HTML tags, extra whitespace, etc.)

### Markdown Rendering

- [ ] Open output .md file in markdown viewer
- [ ] Verify bold speaker names render correctly
- [ ] Verify no unescaped markdown special characters break rendering
- [ ] Verify timestamps (if enabled) render cleanly

## Consolidation Testing

### Same Speaker Consecutive Cues

- [ ] Create VTT with 3+ consecutive cues from same speaker
- [ ] Run conversion
- [ ] Verify single paragraph in output
- [ ] Verify text is joined with single spaces
- [ ] Verify no missing words or extra spaces

### Alternating Speakers

- [ ] Create VTT with A, B, A, B, A, B pattern
- [ ] Run conversion
- [ ] Verify 6 separate paragraphs in output
- [ ] Verify speaker order is preserved

### Sentence Boundaries

- [ ] Create cues ending with terminal punctuation (. ? !)
- [ ] Verify punctuation is preserved
- [ ] Verify next sentence/cue text starts appropriately

## Edge Cases

### Empty VTT File

- [ ] Create file with just "WEBVTT" header and nothing else
- [ ] Run conversion
- [ ] Verify appropriate behavior (empty output or error)

### Single Speaker Throughout

- [ ] Create VTT with only one speaker for entire transcript
- [ ] Run conversion
- [ ] Verify single continuous output (or appropriately broken paragraphs)

### Very Short Cues

- [ ] Create VTT with single-word cues
- [ ] Run conversion
- [ ] Verify words are properly joined

### Empty/Whitespace Cues

- [ ] Create VTT with cues containing only whitespace
- [ ] Run conversion
- [ ] Verify empty cues are skipped

### Malformed Tags

- [ ] Create VTT with unclosed `<v>` tags
- [ ] Create VTT with malformed HTML tags
- [ ] Run conversion
- [ ] Verify parser doesn't crash
- [ ] Verify reasonable output is produced

## Regression Testing

After making any changes to the codebase:
- [ ] Re-run automated test suite: `cargo test`
- [ ] Re-test basic conversion with known-good VTT file
- [ ] Re-verify exit codes for common error cases
- [ ] Re-test at least one flag combination
- [ ] Re-verify Markdown output quality

## Acceptance Criteria

All tests must pass for the feature to be considered complete:
- ✅ All automated tests (unit + integration) pass
- ✅ All platform-specific tests pass on target platforms
- ✅ All error conditions produce correct exit codes and messages
- ✅ All CLI flags work as documented
- ✅ Output Markdown is properly formatted and readable
- ✅ Performance is acceptable (< 5 seconds for large files)
- ✅ No crashes or panics on expected inputs (including malformed files)
