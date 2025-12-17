# Default Include Timestamps

## Overview

This change adds an *opt-in*, per-user default for timestamp inclusion so that conversions which omit `--include-timestamps` (including file-association / double-click runs) can still include timestamps.

The built-in default remains `none` for users who do not configure anything.

## Behavior

### Timestamp inclusion (boolean)

`vtt-to-md` supports a single timestamp inclusion setting:

- `include_timestamps = false`: do not include timestamps in Markdown output
- `include_timestamps = true`: include the first timestamp per consolidated speaker turn

Note: due to cue consolidation, timestamps are per speaker turn (the first cue in that turn).

### Resolution and precedence

When `--include-timestamps[=<BOOL>]` is explicitly provided, that value is always used.

When `--include-timestamps` is omitted, `vtt-to-md` resolves an effective mode using the following precedence (highest → lowest):

1. CLI flag (`--include-timestamps <MODE>`)
2. Environment variable (`VTT_TO_MD_INCLUDE_TIMESTAMPS`)
3. Per-user config file (`config.toml`)
4. Built-in default (`false`)

## Configuration

### Environment variable

Preferred values for `VTT_TO_MD_INCLUDE_TIMESTAMPS`:

- `true`
- `false`

Legacy values are still accepted for a transition period (with a deprecation warning):

- `none` (equivalent to `false`)
- `first` / `each` (equivalent to `true`)

Examples:

- Windows (PowerShell):
  - `setx VTT_TO_MD_INCLUDE_TIMESTAMPS first`
- Linux/macOS (bash/zsh):
  - `export VTT_TO_MD_INCLUDE_TIMESTAMPS=first`

### Per-user config file

Create a TOML file named `config.toml` under the platform config directory in a `vtt-to-md` subfolder.

Locations:

- Windows: `%APPDATA%\vtt-to-md\config.toml`
- Linux: `$XDG_CONFIG_HOME/vtt-to-md/config.toml` (fallback: `~/.config/vtt-to-md/config.toml`)
- macOS: `~/Library/Application Support/vtt-to-md/config.toml`

Supported key:

```toml
include_timestamps = true
```

Legacy string values (deprecated) are still accepted:

```toml
include_timestamps = "first"  # or "each" or "none"
```

## Warning and fallback behavior

Configuration problems are non-fatal.

- If `VTT_TO_MD_INCLUDE_TIMESTAMPS` is set but empty or invalid, the env var is ignored and a warning is printed to stderr.
- If the config file exists but cannot be read, parsed as TOML, or contains an invalid `include_timestamps` value, the config is ignored and a warning is printed to stderr.
- In all cases, conversion continues using the next fallback in the precedence chain.

Warnings are emitted as `Warning: ...` on stderr.

## Usage examples

### Example: Configure a default (no CLI flag)

With `VTT_TO_MD_INCLUDE_TIMESTAMPS=true` (or `include_timestamps = true` in config), running:

```bash
vtt-to-md meeting.vtt
```

will behave the same as:

```bash
vtt-to-md meeting.vtt --include-timestamps
```

### Example: CLI always overrides defaults

Even if you have a default configured, you can explicitly disable timestamps for a single run:

```bash
vtt-to-md meeting.vtt --include-timestamps=false
```

## Testing guide

To manually verify the behavior:

1. Run `vtt-to-md input.vtt --stdout` with no env var and no config file; confirm there are no timestamps.
2. Set `VTT_TO_MD_INCLUDE_TIMESTAMPS=first` and rerun with no `--include-timestamps`; confirm timestamps appear.
3. Keep the env var set and run with `--include-timestamps none`; confirm timestamps do not appear.
4. Set `VTT_TO_MD_INCLUDE_TIMESTAMPS=bogus` and rerun; confirm conversion still succeeds and stderr includes a `Warning:`.

## References

- Project issue: https://github.com/lossyrob/vtt-to-md/issues/16
- Main CLI documentation: `.paw/work/vtt-to-md-cli/Docs.md`
