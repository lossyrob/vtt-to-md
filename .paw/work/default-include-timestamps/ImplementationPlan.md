# Default Include Timestamps — Implementation Plan

## Overview
Implement an opt-in, user-level default for timestamp inclusion so “file-open” (file association / double-click) conversions can include timestamps without requiring extra CLI flags.

This work originally introduced a 3-value timestamp mode (`none|first|each`) configurable via env var and `config.toml`. Because cue consolidation collapses multiple cues into a single speaker turn, `each` does not provide meaningfully different output than `first` (the current implementation displays the first timestamp per consolidated segment).

This updated plan adds a new phase that **simplifies the public interface to a single boolean setting**: “include timestamps”.

Configuration sources (final state after Phase 4):
1. CLI option: `--include-timestamps[=<BOOL>]` (see migration notes for legacy values)
2. Environment variable: `VTT_TO_MD_INCLUDE_TIMESTAMPS=true|false` (legacy `none|first|each` accepted with deprecation warning)
3. User config file: `config.toml` with `include_timestamps = true|false` (legacy string values accepted with deprecation warning)

Precedence is deterministic: **CLI option > environment variable > config file > built-in default**.

## Current State Analysis
- Timestamp behavior is currently controlled by a `TimestampMode` value resolved using precedence across:
  - CLI option `--include-timestamps MODE` (optional)
  - Environment variable `VTT_TO_MD_INCLUDE_TIMESTAMPS`
  - User config file `vtt-to-md/config.toml`
  - Built-in default
- CLI parsing represents “omitted vs explicit” via `Option<TimestampMode>` ([src/cli.rs](src/cli.rs#L70-L100)).
- The resolved `TimestampMode` is passed through the pipeline and consumed by consolidation + markdown formatting in [src/main.rs](src/main.rs#L39-L69).
- Tests already cover the three modes (`none`, `first`, `each`) in unit tests (markdown + consolidation) and integration tests.

Additionally:
- `TimestampMode::Each` currently renders the **first** timestamp per consolidated speaker segment (not per original cue) due to cue consolidation ([src/markdown.rs](src/markdown.rs#L47-L88)).

Key consequence: the `none|first|each` enum surface suggests user-visible behavior that consolidation cannot deliver (“each cue timestamp”), which makes the UX confusing and pushes complexity into configuration and documentation.

## Desired End State
When the user runs `vtt-to-md INPUT` without specifying `--include-timestamps`:
- If `VTT_TO_MD_INCLUDE_TIMESTAMPS` is set to a valid boolean value, it is used.
- Else if the config file exists and contains a valid boolean `include_timestamps`, it is used.
- Else the program behaves exactly as today: timestamps are not included.

When timestamp inclusion is enabled (by CLI/env/config), the output includes **one timestamp per consolidated speaker segment**, representing when the speaker turn began (the first cue timestamp in that turn).

When the user explicitly passes `--include-timestamps` (with or without a boolean value), that explicit CLI choice is always used regardless of env/config.

If env/config is present but invalid/unreadable/unparsable, the conversion continues using the next fallback in precedence order, and the user is informed via stderr.

### Key Discoveries
- CLI option definition: [src/cli.rs](src/cli.rs#L70-L110)
- Pipeline consumption points: [src/main.rs](src/main.rs#L39-L69)
- Consolidation implies `each` ≈ `first`: [src/markdown.rs](src/markdown.rs#L47-L88)
- Existing end-to-end test harness uses `std::process::Command` in [tests/integration_test.rs](tests/integration_test.rs#L1-L80)

## What We’re NOT Doing
- Changing the built-in default timestamp mode for users who do not opt in.
- Adding an MSI installer checkbox/UI to write config.
- Modifying Windows file-association verbs to append CLI flags.
- Adding additional configurable behavior beyond timestamp mode.
- Re-introducing per-cue timestamp output while still consolidating cues into speaker turns.

## Implementation Approach
### Configuration resolution strategy
- Change the CLI model to represent “flag omitted” vs “flag provided” by making `Args.include_timestamps` optional.
- Introduce a small configuration resolver that:
  - Reads `VTT_TO_MD_INCLUDE_TIMESTAMPS`.
  - Resolves the per-user config file location using the platform’s standard config directory and reads `vtt-to-md/config.toml`.
  - Parses only the single key `include_timestamps`.
  - Validates values as boolean and supports legacy string values (`none|first|each`) for backward compatibility.
  - Emits non-fatal warnings to stderr when env/config is present but invalid or unreadable.

### Timestamp behavior simplification
- Replace the `TimestampMode` enum (`none|first|each`) with a single boolean setting.
- Define “include timestamps” precisely as “include the first timestamp per consolidated speaker segment”. This reflects the observable behavior users currently get from both `first` and `each`.

### Backward compatibility strategy
- Maintain the existing env var name (`VTT_TO_MD_INCLUDE_TIMESTAMPS`) and config key (`include_timestamps`).
- Accept legacy values for one transition period:
  - `none` → `false`
  - `first` / `each` → `true`
- Emit a clear deprecation warning when legacy values are detected, guiding users to boolean equivalents.
- Prefer to keep the CLI surface to a single option name (`--include-timestamps`) while still accepting legacy syntaxes where feasible.

### Dependency choice
- Add `dirs` (or equivalent) to locate the per-user config directory consistently across Windows/macOS/Linux.
- Add `toml` to parse `config.toml`.

Rationale: keep the implementation lightweight and avoid reinventing OS-specific config directory rules.

## Phase Summary
1. **Phase 1: Add config resolver + CLI “omitted vs explicit”** — Introduce config reading/parsing and update CLI parsing to support precedence logic.
2. **Phase 2: Apply precedence in conversion pipeline** — Compute an effective timestamp mode once, thread it through consolidation/markdown, and surface warnings.
3. **Phase 3: Tests for precedence + invalid config** — Add integration tests covering env/config precedence and fallback behavior.
4. **Phase 4: Simplify timestamps to a single boolean flag** — Remove the `none|first|each` enum surface, align behavior with consolidation (one timestamp per speaker turn), and migrate/accept legacy config values.

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

## Phase 4: Simplify Timestamps to a Single Boolean Flag

### Overview
Remove the `TimestampMode` enum and expose a single “include timestamps” boolean throughout the application. This phase also updates configuration encoding/parsing (env + `config.toml`) and CLI help/docs to reflect the simplified behavior, while preserving backward compatibility for existing users.

### Changes Required

#### 1) Replace `TimestampMode` with a boolean setting
**Files**:
- src/cli.rs
- src/main.rs
- src/consolidator.rs
- src/markdown.rs

**Changes**:
- Replace the public timestamp setting type from `TimestampMode` to a boolean `include_timestamps`.
- Define semantics: when `include_timestamps=true`, prepend the first timestamp per consolidated speaker segment.
- Remove `TimestampMode::Each` behavior and any user-facing mention of modes.

**Tests**:
- Update existing unit tests in consolidator/markdown to cover `include_timestamps=false` and `include_timestamps=true`.

#### 2) Update CLI option parsing + help text
**Files**:
- src/cli.rs

**Changes**:
- Change CLI surface from `--include-timestamps MODE` to a boolean-friendly option:
  - Support `--include-timestamps` (equivalent to `true`).
  - Support explicit disable via `--include-timestamps=false`.
  - For backward compatibility, accept legacy values (`none|first|each`) as synonyms for boolean values, and emit a deprecation warning directing users to `true|false`.
- Update `--help` text to explain that timestamps are per speaker turn due to consolidation.

**Tests**:
- Add/adjust integration tests to verify legacy CLI values are accepted (where applicable) and warnings are emitted.

#### 3) Migrate env var + config file encoding/parsing
**Files**:
- src/config.rs
- README.md
- CHANGELOG.md

**Changes**:
- Keep env var name `VTT_TO_MD_INCLUDE_TIMESTAMPS`, but parse it as boolean with accepted values like `true|false|1|0|yes|no|on|off`.
- Continue accepting legacy env values (`none|first|each`) with a deprecation warning.
- Change `config.toml` key `include_timestamps` to prefer boolean (`true|false`).
- Continue accepting legacy string values (`"none"|"first"|"each"`) with a deprecation warning.
- Ensure precedence remains `CLI > env > config > default`.

**Tests**:
- Extend config parsing unit tests to cover:
  - boolean values
  - legacy string values
  - invalid values
- Extend integration tests to cover legacy env/config values and verify warning + correct effective behavior.

### Success Criteria

#### Automated Verification
- [x] `cargo test` passes on Windows.
- [x] `vtt-to-md --help` documents the boolean option (no `none|first|each` listed as primary behavior).
- [x] Existing precedence behavior remains correct for boolean settings.

#### Manual Verification
- [ ] With `include_timestamps=true` in `config.toml`, running `vtt-to-md input.vtt --stdout` includes timestamps.
- [ ] With the same config, running `vtt-to-md input.vtt --include-timestamps=false --stdout` produces output without timestamps.
- [ ] Legacy values in env/config still work but clearly warn.

**Status:** Completed.

**Notes:**
- Replaced `TimestampMode` with a single boolean `include_timestamps` threaded through CLI → main → consolidator → markdown.
- CLI now supports `--include-timestamps` / `--include-timestamps=false`; legacy `none|first|each` are accepted with a deprecation warning.
- Env/config prefer booleans (`true|false`) while accepting legacy strings (`none|first|each`) with a deprecation warning.
- Consolidation/markdown behavior is now explicitly “first timestamp per consolidated speaker segment” when enabled.

## Cross-Phase Testing Strategy

### Integration Scenarios
- No CLI option + no env + no config → built-in default (timestamps not included).
- No CLI option + env set → env wins.
- No CLI option + config set → config wins.
- CLI option provided + env/config set → CLI wins.
- Legacy values in env/config are accepted and map to the equivalent boolean behavior.

### Manual Testing Steps
1. Create config file at the platform path and set `include_timestamps = true`.
2. Run `vtt-to-md some.vtt --stdout` and verify timestamps appear.
3. Run `vtt-to-md some.vtt --include-timestamps=false --stdout` and verify timestamps do not appear.

## Performance Considerations
- Config and env reading happens once per run and is negligible compared to file I/O and parsing.

## Migration Notes
- Config/env format migration:
  - Preferred encoding becomes boolean (`true|false`) for both env and `config.toml`.
  - Legacy string values (`none|first|each`) remain accepted for a transition period and will emit a deprecation warning.

- CLI migration:
  - Primary syntax becomes `--include-timestamps` / `--include-timestamps=false`.
  - Legacy `--include-timestamps none|first|each` should be accepted where feasible and mapped to boolean equivalents with a deprecation warning.

- Behavioral clarification:
  - The “each” concept is removed from the public interface. With consolidation enabled, timestamps are per speaker turn (first cue in the turn).

## References
- Issue: https://github.com/lossyrob/vtt-to-md/issues/16
- Spec: .paw/work/default-include-timestamps/Spec.md
- Research: .paw/work/default-include-timestamps/SpecResearch.md, .paw/work/default-include-timestamps/CodeResearch.md
- Related code entrypoints: [src/cli.rs](src/cli.rs#L75-L104), [src/main.rs](src/main.rs#L39-L69)
