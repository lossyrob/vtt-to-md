# Spec Research: Default Include Timestamps

## Summary
No external research is required beyond the feature specification in `Spec.md`.

## Confirmed Requirements (from Spec)
- Precedence must be deterministic and documented: `CLI flag > environment variable > configuration file > built-in default`.
- Built-in default must remain `none` for users who do not opt in.
- Supported values are exactly: `none`, `first`, `each`.
- Env var name and config format:
  - Env: `VTT_TO_MD_INCLUDE_TIMESTAMPS=none|first|each`
  - Config: `config.toml` with `include_timestamps = "none"|"first"|"each"`
- Config locations:
  - Windows: `%APPDATA%\vtt-to-md\config.toml`
  - macOS: `~/Library/Application Support/vtt-to-md/config.toml`
  - Linux: `$XDG_CONFIG_HOME/vtt-to-md/config.toml` (fallback `~/.config/vtt-to-md/config.toml`)

## Notes / Constraints
- “File association / double-click runs” are not reliably distinguishable from normal CLI invocations (both typically pass just the input file path). The spec explicitly allows applying the configured default to any run where `--include-timestamps` is omitted.
