# Default Include Timestamps — Implementation Plan

## Overview
Implement an opt-in, user-level default for `--include-timestamps` so “file-open” (file association / double-click) conversions can include timestamps without requiring extra CLI flags. The built-in default remains `none` unless the user opts in.

This plan adds two configuration sources:
1. Environment variable `VTT_TO_MD_INCLUDE_TIMESTAMPS=none|first|each`
2. User config file `config.toml` with `include_timestamps = "none"|"first"|"each"`

Precedence must be deterministic: **CLI flag > environment variable > config file > built-in default**.

## Current State Analysis
- Timestamp behavior is currently controlled only via the CLI flag `--include-timestamps MODE` with clap `default_value = "none"` in [src/cli.rs](src/cli.rs#L75-L90).
- The parsed `TimestampMode` is passed unchanged through the pipeline and consumed by consolidation + markdown formatting in [src/main.rs](src/main.rs#L39-L69).
- Tests already cover the three modes (`none`, `first`, `each`) in unit tests (markdown + consolidation) and integration tests for `first`/`each` flags.

Key consequence: because clap currently sets a default value for `include_timestamps`, the program cannot distinguish “user omitted the flag” vs “user explicitly set `none`”. That distinction is required for correctly applying configured defaults.

## Desired End State
When the user runs `vtt-to-md INPUT` without specifying `--include-timestamps`:
- If `VTT_TO_MD_INCLUDE_TIMESTAMPS` is set to a valid value, it is used.
- Else if the config file exists and contains a valid `include_timestamps` value, it is used.
- Else the program behaves exactly as today: timestamps are not included (`none`).

When the user explicitly passes `--include-timestamps MODE`, that value is always used regardless of env/config.

If env/config is present but invalid/unreadable/unparsable, the conversion continues using the next fallback in precedence order, and the user is informed via stderr.

### Key Discoveries
- CLI flag definition + default: [src/cli.rs](src/cli.rs#L75-L90)
- Pipeline consumption points: [src/main.rs](src/main.rs#L39-L69)
- Existing end-to-end test harness uses `std::process::Command` in [tests/integration_test.rs](tests/integration_test.rs#L1-L80)

## What We’re NOT Doing
- Changing the built-in default timestamp mode for users who do not opt in.
- Adding an MSI installer checkbox/UI to write config.
- Modifying Windows file-association verbs to append CLI flags.
- Adding additional configurable behavior beyond timestamp mode.

## Implementation Approach
### Configuration resolution strategy
- Change the CLI model to represent “flag omitted” vs “flag provided” by making `Args.include_timestamps` optional.
- Introduce a small configuration resolver that:
  - Reads `VTT_TO_MD_INCLUDE_TIMESTAMPS`.
  - Resolves the per-user config file location using the platform’s standard config directory and reads `vtt-to-md/config.toml`.
  - Parses only the single key `include_timestamps`.
  - Validates values against the same allowed set as the CLI (`none`, `first`, `each`).
  - Emits non-fatal warnings to stderr when env/config is present but invalid or unreadable.

### Dependency choice
- Add `dirs` (or equivalent) to locate the per-user config directory consistently across Windows/macOS/Linux.
- Add `toml` to parse `config.toml`.

Rationale: keep the implementation lightweight and avoid reinventing OS-specific config directory rules.

## Phase Summary
1. **Phase 1: Add config resolver + CLI “omitted vs explicit”** — Introduce config reading/parsing and update CLI parsing to support precedence logic.
2. **Phase 2: Apply precedence in conversion pipeline** — Compute an effective timestamp mode once, thread it through consolidation/markdown, and surface warnings.
3. **Phase 3: Tests for precedence + invalid config** — Add integration tests covering env/config precedence and fallback behavior.

---

## Phase 1: Add Config Resolver + CLI “Omitted vs Explicit”

### Overview
Create a focused configuration module and adjust CLI parsing so the program can detect whether `--include-timestamps` was explicitly provided.

### Changes Required

#### 1) CLI model update
**Files**:
- src/cli.rs

**Changes**:
- Change `Args.include_timestamps` from `TimestampMode` to `Option<TimestampMode>`.
- Remove the clap `default_value = "none"` so omission results in `None`.
- Ensure `--include-timestamps none` remains supported (this becomes `Some(TimestampMode::None)`).

**Tests**:
- Prefer integration-level verification (Phase 3) to ensure behavioral equivalence.

#### 2) Timestamp mode parsing helper
**Files**:
- src/cli.rs (or a small helper module co-located with config)

**Changes**:
- Add a single canonical parser used by env/config reading. Options:
  1. Implement `FromStr` for `TimestampMode` (case-insensitive) and reuse it.
  2. Add a dedicated `parse_timestamp_mode(&str) -> Option<TimestampMode>` helper.

**Decision**: choose whichever best matches existing style; keep it small and non-allocating when possible.

**Tests**:
- Unit tests for parsing should cover `none|first|each`, case variations, and invalid strings.

#### 3) New config module for resolving defaults
**Files**:
- src/config.rs (new)
- src/main.rs (module wiring)
- Cargo.toml (dependencies)

**Changes**:
- Add `mod config;` in main.
- Implement a resolver with a stable interface, e.g.:
  - `config::resolve_default_timestamp_mode() -> (Option<TimestampMode>, Vec<String>)`
    - Returns the best configured default (env > file) and any warning strings.
- Implement config file discovery using per-user config directory:
  - Windows: `%APPDATA%\vtt-to-md\config.toml`
  - macOS: `~/Library/Application Support/vtt-to-md/config.toml`
  - Linux: `$XDG_CONFIG_HOME/vtt-to-md/config.toml` (fallback `~/.config/vtt-to-md/config.toml`)

**Tests**:
- Unit test `config::parse_config_value` (string-only), avoiding filesystem dependency.

### Success Criteria

#### Automated Verification
- [x] `cargo test` passes.
- [x] CLI still accepts `--include-timestamps first|each|none`.

#### Manual Verification
- [ ] `vtt-to-md input.vtt --help` still clearly documents timestamp modes.

**Status:** Completed.

**Notes:**
- `--include-timestamps` is now optional (no clap default) so “flag omitted” can be distinguished from `--include-timestamps none`.
- Added shared parsing via `TimestampMode::from_str` and unit tests.
- Added config/env resolver in `src/config.rs`.

---

## Phase 2: Apply Precedence in the Conversion Pipeline

### Overview
Compute an “effective” timestamp mode once per run using precedence rules and pass it through the conversion pipeline.

### Changes Required

#### 1) Compute effective mode with precedence
**Files**:
- src/main.rs
- src/config.rs

**Changes**:
- Introduce `effective_timestamp_mode` computed as:
  1. If `args.include_timestamps.is_some()`: use that value.
  2. Else if env var resolves to a valid mode: use that.
  3. Else if config file resolves to a valid mode: use that.
  4. Else use built-in default `TimestampMode::None`.
- Print any non-fatal warnings to stderr exactly once (e.g., before conversion begins).

**Notes**:
- Favor computing once (not at every call site) to avoid duplicated warnings.

#### 2) Thread effective mode through pipeline
**Files**:
- src/main.rs

**Changes**:
- Update `run_conversion` to accept the effective mode explicitly, or store it in `Args` during validation.
- Pass the effective mode to:
  - `consolidator::consolidate_cues(..., timestamp_mode)`
  - `markdown::format_markdown(..., timestamp_mode)`

**Tests**:
- Covered primarily in Phase 3 integration tests.

### Success Criteria

#### Automated Verification
- [x] `cargo test` passes.
- [x] Existing integration tests for `--include-timestamps first|each` still pass unchanged.

#### Manual Verification
- [ ] With `VTT_TO_MD_INCLUDE_TIMESTAMPS=first`, running `vtt-to-md input.vtt --stdout` includes timestamps.
- [ ] With env set to `first`, running `vtt-to-md input.vtt --include-timestamps none --stdout` produces output without timestamps.

**Status:** Completed.

**Notes:**
- Precedence implemented as `CLI > env > config > built-in default (none)`.
- Non-fatal configuration problems emit `Warning:` to stderr once.

---

## Phase 3: Tests for Precedence + Invalid Config

### Overview
Add integration test coverage for env/config precedence and for graceful fallback when invalid configuration is present.

### Changes Required

#### 1) Integration tests for env var default
**Files**:
- tests/integration_test.rs

**Changes**:
- Add a test that sets `VTT_TO_MD_INCLUDE_TIMESTAMPS=first` and runs the binary without `--include-timestamps`.
- Assert stdout includes a timestamp prefix like `[00:00:00.000]`.

#### 2) Integration tests for config file default
**Files**:
- tests/integration_test.rs

**Changes**:
- Create a temporary directory and point the process’s per-user config root at it (platform-dependent):
  - On Windows: set `APPDATA` to the temp dir.
  - On Linux: set `XDG_CONFIG_HOME` to the temp dir.
  - On macOS: set `HOME` to the temp dir (and place the file under `Library/Application Support/...`).
- Write `vtt-to-md/config.toml` with `include_timestamps = "each"`.
- Run without `--include-timestamps` and without `VTT_TO_MD_INCLUDE_TIMESTAMPS`.
- Assert stdout includes a timestamp prefix.

#### 3) Integration tests for precedence + invalid values
**Files**:
- tests/integration_test.rs

**Changes**:
- CLI overrides env:
  - Set env to `first`, pass `--include-timestamps none`, assert no timestamp prefix.
- Invalid env falls back to config/built-in:
  - Set env to `bogus`, ensure conversion still succeeds and stderr contains a warning.
- Invalid config falls back to built-in:
  - Write `include_timestamps = "bogus"`, ensure conversion succeeds, no timestamps in output, stderr contains a warning.

### Success Criteria

#### Automated Verification
- [x] `cargo test` passes on Windows.

#### Manual Verification
- [ ] Double-click / file association run (or equivalent “no flags” invocation) picks up a configured default.

**Status:** Completed.

**Notes:**
- Integration tests isolate user env/config and add Windows-only coverage for env/config precedence and invalid values.

---

## Cross-Phase Testing Strategy

### Integration Scenarios
- No flags + no env + no config → built-in default (`none`).
- No flags + env set → env wins.
- No flags + config set → config wins.
- Flag provided + env/config set → flag wins.

### Manual Testing Steps
1. Create config file at the platform path and set `include_timestamps = "first"`.
2. Run `vtt-to-md some.vtt --stdout` and verify timestamps appear.
3. Run `vtt-to-md some.vtt --include-timestamps none --stdout` and verify timestamps do not appear.

## Performance Considerations
- Config and env reading happens once per run and is negligible compared to file I/O and parsing.

## Migration Notes
- No migrations. This is additive behavior controlled by user opt-in.

## References
- Issue: https://github.com/lossyrob/vtt-to-md/issues/16
- Spec: .paw/work/default-include-timestamps/Spec.md
- Research: .paw/work/default-include-timestamps/SpecResearch.md, .paw/work/default-include-timestamps/CodeResearch.md
- Related code entrypoints: [src/cli.rs](src/cli.rs#L75-L104), [src/main.rs](src/main.rs#L39-L69)
