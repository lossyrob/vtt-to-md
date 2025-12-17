# Feature Specification: Default Include Timestamps (Configurable for File-Open Runs)

**Branch**: auto  |  **Created**: 2025-12-17  |  **Status**: Draft
**Input Brief**: Make `--include-timestamps first` configurable as the default for file-association / double-click conversions, without changing the built-in default for everyone.

## Overview
A common workflow for `vtt-to-md` is opening a `.vtt` file directly from the OS (for example via file association or double-click), letting the tool convert it to Markdown without the user typing a command. Today, timestamp inclusion is an explicit choice made through a CLI flag, which works well when a user is running commands manually but is awkward when the program is launched by the OS.

This feature adds an opt-in user-level default for timestamp inclusion so that file-open conversions can include timestamps automatically. Users who want timestamps (especially “first timestamp per speaker turn”) can enable that preference once and have it applied whenever they convert a file without specifying a timestamp mode.

The change must preserve existing behavior for users who do nothing: if no configuration is set, the built-in default remains `none`. When configuration is set, it only applies when the user did not explicitly choose a timestamp mode on the command line; explicit CLI choices must always win.

Documentation will describe how to enable the default, how configuration is discovered on each platform, and the precedence rules so users can predict the final behavior.

## Objectives
- Enable an opt-in default timestamp mode for conversions started via file association / double-click (Rationale: users cannot easily append CLI flags in that workflow).
- Preserve the current built-in default behavior unless the user opts in (Rationale: avoids surprising existing users).
- Provide clear precedence rules so explicit CLI intent always wins (Rationale: prevents “mysterious” behavior changes).
- Document the feature with simple, copy/paste-friendly examples.

## User Scenarios & Testing
### User Story P1 – Opt-in default timestamps for file-open conversions
Narrative: A user frequently double-clicks `.vtt` files to produce Markdown and wants timestamps to be included without having to rerun conversions from a terminal.
Independent Test: Configure the default mode to `first`, then open a `.vtt` file via file association and observe timestamped output.
Acceptance Scenarios:
1. Given a user has configured default timestamp mode to `first`, When they convert a file without specifying `--include-timestamps`, Then the output includes timestamps using the `first` mode.
2. Given no user configuration exists, When they convert a file without specifying `--include-timestamps`, Then the output uses the built-in default timestamp mode (`none`).

### User Story P1 – Explicit CLI overrides configuration
Narrative: A user has enabled default timestamps but occasionally wants a “clean” transcript.
Independent Test: With a default set to `first`, run a conversion with `--include-timestamps none` and verify no timestamps are present.
Acceptance Scenarios:
1. Given a user has configured default timestamp mode to `first`, When they run a conversion with `--include-timestamps none`, Then the output contains no timestamps.
2. Given a user has configured default timestamp mode to `first`, When they run a conversion with `--include-timestamps each`, Then the output includes timestamps using the `each` mode.

### Edge Cases
- If the configuration value is present but invalid, the conversion proceeds using the built-in default (`none`) and the user is informed of the invalid setting.
- If a configuration file exists but cannot be read or parsed, the conversion proceeds using the built-in default (`none`) and the user is informed of the problem.
- If both an environment variable and a config file are present, the environment variable is used.

## Requirements
### Functional Requirements
- FR-001: The program supports a user-level default timestamp mode that is applied when the user does not explicitly provide `--include-timestamps`. (Stories: P1)
- FR-001a: A “file-open conversion” includes file association / double-click runs and manual CLI runs where the user provides an input file but omits `--include-timestamps`; in these cases, the configured default may apply. (Stories: P1)
- FR-002: `--include-timestamps MODE` always overrides any configured default. (Stories: P1)
- FR-003: The supported configured modes are exactly the same as the CLI-supported modes: `none`, `first`, `each`. (Stories: P1)
- FR-004: The program supports configuring the default timestamp mode via an environment variable. (Stories: P1)
- FR-005: The program supports configuring the default timestamp mode via a user-scope configuration file. (Stories: P1)
- FR-006: Precedence is deterministic and documented: CLI flag > environment variable > configuration file > built-in default. (Stories: P1)
- FR-007: If configuration is missing, unreadable, unparsable, or invalid, the program falls back to the built-in default and continues processing. (Stories: P1)

### Key Entities
- Timestamp inclusion mode: One of `none`, `first`, `each`.

### Cross-Cutting / Non-Functional
- Usability: Configuration mechanisms must be simple (single value) and not require elevated privileges.
- Compatibility: Default behavior remains unchanged when no configuration is present.

## Success Criteria
- SC-001: A user can enable default timestamps for file-open conversions by setting either a config file value or an environment variable. (FR-001, FR-004, FR-005)
- SC-002: When both env and config are set, the env value takes effect. (FR-006)
- SC-003: When a user explicitly passes `--include-timestamps none`, timestamps are not included even if a default is configured. (FR-002)
- SC-004: With no configuration present, conversions behave exactly as they do today with respect to timestamp inclusion. (FR-007)
- SC-005: README documents how to enable the default and the precedence rules. (FR-006)

## Proposed Configuration
### Environment Variable
- Name: `VTT_TO_MD_INCLUDE_TIMESTAMPS`
- Values: `none` | `first` | `each`

### Configuration File
- File name: `config.toml`
- Key: `include_timestamps`
- Values: `"none"` | `"first"` | `"each"`

Example setting: key `include_timestamps` set to the string value `first`.

## Platform-Specific Config Locations
The program looks for a user-scope config file at the standard per-platform configuration directory.

- Windows: `%APPDATA%\vtt-to-md\config.toml`
- macOS: `~/Library/Application Support/vtt-to-md/config.toml`
- Linux: `$XDG_CONFIG_HOME/vtt-to-md/config.toml` (fallback to `~/.config/vtt-to-md/config.toml`)

## Scope
In Scope:
- Add opt-in configuration mechanisms (env var and config file) to provide a default timestamp mode.
- Apply configured default when the user does not explicitly pass `--include-timestamps`.
- Document the new behavior and precedence rules.

Out of Scope:
- Changing the built-in default for users who do not opt in.
- Adding an interactive MSI installer checkbox/UI for this setting.
- Writing or modifying user config during installation.

## Dependencies
- None beyond standard access to the user’s environment variables and per-user config directory.

## Risks & Mitigations
- Misconfiguration leads to confusion: Mitigation: clear README documentation, explicit precedence rules, and a clear message when config/env value is invalid.
- Users expect “file association only” behavior: Mitigation: documentation clarifies that the default applies whenever the flag is omitted; explicit CLI usage can always override.

## References
- Issue: https://github.com/lossyrob/vtt-to-md/issues/16
- External: None
