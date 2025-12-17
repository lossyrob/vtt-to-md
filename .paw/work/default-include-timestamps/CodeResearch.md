---
date: 2025-12-17T12:18:12-05:00
git_commit: 20771462c184cd516cdc5e30b7b5951689ea85e1
branch: feature/16-include-timestamps-default-association
repository: vtt-to-md
topic: "Default Include Timestamps"
tags: [research, cli, timestamps, clap, markdown]
status: complete
last_updated: 2025-12-17
---

# Research: Default Include Timestamps

**Date**: 2025-12-17 12:18:12 -05:00
**Git Commit**: 20771462c184cd516cdc5e30b7b5951689ea85e1
**Branch**: feature/16-include-timestamps-default-association
**Repository**: vtt-to-md

## Research Question
Where does `vtt-to-md` currently parse and apply `--include-timestamps`, what are the supported modes and defaults, and where are the existing tests and documentation for timestamp output behavior?

## Summary
Timestamp inclusion is currently a CLI-only setting parsed into `Args.include_timestamps` as a `TimestampMode` with a clap default of `none` ([src/cli.rs](src/cli.rs#L83-L90)). The parsed mode is passed through the conversion pipeline unchanged and is consumed by both speaker consolidation and Markdown formatting ([src/main.rs](src/main.rs#L39-L69)). Unit tests in the consolidation and markdown modules cover the three modes, and integration tests cover the `first` and `each` flags via end-to-end execution of the compiled binary ([tests/integration_test.rs](tests/integration_test.rs#L174-L215)).

## Detailed Findings

### CLI surface area: `--include-timestamps`
- CLI arguments are defined in the `Args` struct, derived via clap ([src/cli.rs](src/cli.rs#L12-L91)).
- `include_timestamps` is a long option with `value_name = "MODE"` and `default_value = "none"` ([src/cli.rs](src/cli.rs#L83-L90)).
- The supported modes come from the `TimestampMode` enum, derived as a clap `ValueEnum` ([src/cli.rs](src/cli.rs#L94-L104)).

### Entry point and conversion pipeline
- The program parses CLI arguments with `Args::try_parse()` and then runs `args.validate()` before calling `run_conversion(&args)` ([src/main.rs](src/main.rs#L13-L32)).
- `run_conversion` passes `args.include_timestamps` into speaker consolidation and then into Markdown formatting ([src/main.rs](src/main.rs#L39-L69)).

### Consolidation: how timestamps are stored on segments
- Consolidation produces `SpeakerSegment` values that include:
  - `timestamp: Option<String>` (documented as used by `TimestampMode::First`) ([src/consolidator.rs](src/consolidator.rs#L36-L37))
  - `timestamps: Vec<String>` (documented as used by `TimestampMode::Each`) ([src/consolidator.rs](src/consolidator.rs#L38-L41))
- The consolidation function signature includes `timestamp_mode: TimestampMode` ([src/consolidator.rs](src/consolidator.rs#L70-L74)).
- When a speaker turn is closed, the segment `timestamp` field is set based on the mode:
  - `None` → `None`
  - `First` → uses the first cue timestamp
  - `Each` → `None` (timestamps are retained in the `timestamps` vector)
  ([src/consolidator.rs](src/consolidator.rs#L100-L104), [src/consolidator.rs](src/consolidator.rs#L133-L136)).

### Markdown formatting: how modes render
- Markdown generation is performed by `format_markdown(segments, timestamp_mode)` ([src/markdown.rs](src/markdown.rs#L43-L45)).
- `format_markdown` matches on `timestamp_mode` to determine whether to prepend a timestamp string in each rendered paragraph ([src/markdown.rs](src/markdown.rs#L47-L76)).
- In `TimestampMode::Each`, formatting uses the first element of `segment.timestamps` (if present) as the displayed timestamp prefix for that speaker segment ([src/markdown.rs](src/markdown.rs#L61-L74)).

### Tests covering timestamp behavior
- Integration tests run the compiled `vtt-to-md` binary and verify timestamp output for:
  - `--include-timestamps first` ([tests/integration_test.rs](tests/integration_test.rs#L174-L197))
  - `--include-timestamps each` ([tests/integration_test.rs](tests/integration_test.rs#L199-L222))
- Unit tests exist in the markdown module for all three modes, including `TimestampMode::Each` ([src/markdown.rs](src/markdown.rs#L166-L238)).
- Unit tests exist in the consolidator module for timestamp mode behavior (`none`, `first`, `each`) ([src/consolidator.rs](src/consolidator.rs#L338-L408)).

### User-facing documentation
- README includes an example invocation using `--include-timestamps first` ([README.md](README.md#L45)).
- README lists `--include-timestamps MODE` and describes the default as `none` ([README.md](README.md#L59)).

## Code References
- [src/cli.rs](src/cli.rs#L83-L104) - `Args.include_timestamps` definition and `TimestampMode` clap `ValueEnum`.
- [src/main.rs](src/main.rs#L39-L69) - Conversion pipeline uses `args.include_timestamps` for consolidation + formatting.
- [src/consolidator.rs](src/consolidator.rs#L70-L143) - `consolidate_cues` and how it sets `SpeakerSegment.timestamp` and `SpeakerSegment.timestamps`.
- [src/markdown.rs](src/markdown.rs#L43-L76) - `format_markdown` match on `TimestampMode`.
- [tests/integration_test.rs](tests/integration_test.rs#L174-L222) - End-to-end tests for `first` and `each` CLI modes.
- [README.md](README.md#L59) - CLI docs for `--include-timestamps` and default.

## Open Questions
- None for documenting current behavior; configuration defaults are not part of the current implementation surface described above.
