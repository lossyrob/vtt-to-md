//! Configuration defaults resolution.
//!
//! This module supports an opt-in, user-level default for timestamp inclusion.
//!
//! Supported sources (highest to lowest precedence):
//! 1. Environment variable `VTT_TO_MD_INCLUDE_TIMESTAMPS=true|false` (legacy `none|first|each` accepted)
//! 2. Per-user config file `vtt-to-md/config.toml` with `include_timestamps = true|false` (legacy string values accepted)
//!
//! The built-in default remains `false` when no configured default is present.

use crate::cli::IncludeTimestampsSetting;
use std::fs;
use std::path::PathBuf;
use std::str::FromStr;

const ENV_INCLUDE_TIMESTAMPS: &str = "VTT_TO_MD_INCLUDE_TIMESTAMPS";
const ENV_REMOVE_FILLERS: &str = "VTT_TO_MD_REMOVE_FILLERS";
const APP_DIR_NAME: &str = "vtt-to-md";
const CONFIG_FILE_NAME: &str = "config.toml";
const CONFIG_KEY_INCLUDE_TIMESTAMPS: &str = "include_timestamps";
const CONFIG_KEY_REMOVE_FILLERS: &str = "remove_fillers";

/// Resolve a configured default `include_timestamps` setting (env var > config file).
///
/// This is only intended to be used when the CLI flag `--include-timestamps` was
/// omitted, so that explicit CLI usage always wins.
///
/// Returns a tuple of:
/// - `Option<bool>`: the configured default, if any
/// - `Vec<String>`: warnings suitable for printing to stderr
pub(crate) fn resolve_default_include_timestamps() -> (Option<bool>, Vec<String>) {
    let mut warnings = Vec::new();

    // 1) Environment variable (highest non-CLI precedence)
    if let Ok(raw_value) = std::env::var(ENV_INCLUDE_TIMESTAMPS) {
        let value = raw_value.trim();
        if value.is_empty() {
            warnings.push(format!(
                "{ENV_INCLUDE_TIMESTAMPS} is set but empty; ignoring"
            ));
        } else {
            match IncludeTimestampsSetting::from_str(value) {
                Ok(setting) => {
                    if setting.is_legacy {
                        warnings.push(format!(
                            "{ENV_INCLUDE_TIMESTAMPS} uses legacy value '{value}'; please migrate to true/false"
                        ));
                    }
                    return (Some(setting.value), warnings);
                }
                Err(err) => warnings.push(format!(
                    "invalid {ENV_INCLUDE_TIMESTAMPS}='{value}': {err}; ignoring",
                )),
            }
        }
    }

    // 2) Config file
    let Some(config_path) = config_file_path() else {
        return (None, warnings);
    };

    if !config_path.exists() {
        return (None, warnings);
    }

    let contents = match fs::read_to_string(&config_path) {
        Ok(contents) => contents,
        Err(err) => {
            warnings.push(format!(
                "failed to read config file '{}': {err}; ignoring",
                config_path.display()
            ));
            return (None, warnings);
        }
    };

    match parse_config_include_timestamps(&contents) {
        Ok((value, legacy_warning)) => {
            if let Some(warning) = legacy_warning {
                warnings.push(warning);
            }
            (value, warnings)
        }
        Err(err) => {
            warnings.push(format!(
                "invalid config file '{}': {err}; ignoring",
                config_path.display()
            ));
            (None, warnings)
        }
    }
}

/// Resolve a configured default `remove_fillers` setting (env var > config file).
///
/// Only used when the CLI flag `--remove-fillers` was not provided.
///
/// Returns a tuple of:
/// - `Option<bool>`: the configured default, if any
/// - `Vec<String>`: warnings suitable for printing to stderr
pub(crate) fn resolve_default_remove_fillers() -> (Option<bool>, Vec<String>) {
    let mut warnings = Vec::new();

    // 1) Environment variable
    if let Ok(raw_value) = std::env::var(ENV_REMOVE_FILLERS) {
        let value = raw_value.trim();
        if value.is_empty() {
            warnings.push(format!(
                "{ENV_REMOVE_FILLERS} is set but empty; ignoring"
            ));
        } else {
            match parse_bool_value(value) {
                Some(b) => return (Some(b), warnings),
                None => warnings.push(format!(
                    "invalid {ENV_REMOVE_FILLERS}='{value}' (expected true/false); ignoring"
                )),
            }
        }
    }

    // 2) Config file
    let Some(config_path) = config_file_path() else {
        return (None, warnings);
    };

    if !config_path.exists() {
        return (None, warnings);
    }

    let contents = match fs::read_to_string(&config_path) {
        Ok(contents) => contents,
        Err(err) => {
            warnings.push(format!(
                "failed to read config file '{}': {err}; ignoring",
                config_path.display()
            ));
            return (None, warnings);
        }
    };

    match parse_config_remove_fillers(&contents) {
        Ok(value) => (value, warnings),
        Err(err) => {
            warnings.push(format!(
                "invalid config file '{}': {err}; ignoring",
                config_path.display()
            ));
            (None, warnings)
        }
    }
}

fn parse_bool_value(s: &str) -> Option<bool> {
    match s.trim().to_ascii_lowercase().as_str() {
        "true" | "1" | "yes" | "y" | "on" => Some(true),
        "false" | "0" | "no" | "n" | "off" => Some(false),
        _ => None,
    }
}

fn parse_config_remove_fillers(contents: &str) -> Result<Option<bool>, String> {
    let value: toml::Value = contents
        .parse()
        .map_err(|err| format!("TOML parse error: {err}"))?;

    let Some(rf_value) = value.get(CONFIG_KEY_REMOVE_FILLERS) else {
        return Ok(None);
    };

    if let Some(b) = rf_value.as_bool() {
        return Ok(Some(b));
    }

    if let Some(s) = rf_value.as_str() {
        return parse_bool_value(s)
            .map(|b| Some(b))
            .ok_or_else(|| format!("invalid {CONFIG_KEY_REMOVE_FILLERS}='{s}' (expected true/false)"));
    }

    Err(format!(
        "'{CONFIG_KEY_REMOVE_FILLERS}' must be a boolean or string"
    ))
}

fn config_file_path() -> Option<PathBuf> {
    let base_dir = if cfg!(target_os = "windows") {
        std::env::var_os("APPDATA").map(PathBuf::from)
    } else if cfg!(target_os = "linux") {
        std::env::var_os("XDG_CONFIG_HOME").map(PathBuf::from)
    } else if cfg!(target_os = "macos") {
        std::env::var_os("HOME")
            .map(PathBuf::from)
            .map(|home| home.join("Library").join("Application Support"))
    } else {
        None
    }
    .or_else(dirs::config_dir);

    base_dir.map(|dir| dir.join(APP_DIR_NAME).join(CONFIG_FILE_NAME))
}

fn parse_config_include_timestamps(
    contents: &str,
) -> Result<(Option<bool>, Option<String>), String> {
    let value: toml::Value = contents
        .parse()
        .map_err(|err| format!("TOML parse error: {err}"))?;

    let Some(include_value) = value.get(CONFIG_KEY_INCLUDE_TIMESTAMPS) else {
        return Ok((None, None));
    };

    if let Some(include_bool) = include_value.as_bool() {
        return Ok((Some(include_bool), None));
    }

    let Some(include_str) = include_value.as_str() else {
        return Err(format!(
            "'{CONFIG_KEY_INCLUDE_TIMESTAMPS}' must be a boolean or string"
        ));
    };

    let parsed = IncludeTimestampsSetting::from_str(include_str)
        .map_err(|err| format!("invalid {CONFIG_KEY_INCLUDE_TIMESTAMPS}='{include_str}': {err}"))?;

    let legacy_warning = if parsed.is_legacy {
        Some(format!(
            "config include_timestamps uses legacy value '{include_str}'; please migrate to true/false"
        ))
    } else {
        None
    };

    Ok((Some(parsed.value), legacy_warning))
}

#[cfg(test)]
mod tests {
    use super::parse_config_include_timestamps;
    use super::parse_config_remove_fillers;

    #[test]
    fn parse_config_include_timestamps_missing_key_returns_none() {
        let contents = "other = 'x'\n";
        assert_eq!(
            parse_config_include_timestamps(contents).unwrap(),
            (None, None)
        );
    }

    #[test]
    fn parse_config_include_timestamps_accepts_boolean_values() {
        let contents = "include_timestamps = true\n";
        assert_eq!(
            parse_config_include_timestamps(contents).unwrap(),
            (Some(true), None)
        );
    }

    #[test]
    fn parse_config_include_timestamps_accepts_legacy_string_values_with_warning() {
        let contents = "include_timestamps = \"each\"\n";
        let (value, warning) = parse_config_include_timestamps(contents).unwrap();
        assert_eq!(value, Some(true));
        assert!(warning.is_some());
    }

    #[test]
    fn parse_config_include_timestamps_rejects_invalid_value() {
        let contents = "include_timestamps = \"bogus\"\n";
        assert!(parse_config_include_timestamps(contents).is_err());
    }

    #[test]
    fn parse_config_include_timestamps_rejects_non_bool_non_string() {
        let contents = "include_timestamps = 123\n";
        assert!(parse_config_include_timestamps(contents).is_err());
    }

    #[test]
    fn parse_config_remove_fillers_missing_key_returns_none() {
        let contents = "other = 'x'\n";
        assert_eq!(parse_config_remove_fillers(contents).unwrap(), None);
    }

    #[test]
    fn parse_config_remove_fillers_accepts_boolean() {
        let contents = "remove_fillers = true\n";
        assert_eq!(parse_config_remove_fillers(contents).unwrap(), Some(true));

        let contents = "remove_fillers = false\n";
        assert_eq!(parse_config_remove_fillers(contents).unwrap(), Some(false));
    }

    #[test]
    fn parse_config_remove_fillers_accepts_string_boolean() {
        let contents = "remove_fillers = \"true\"\n";
        assert_eq!(parse_config_remove_fillers(contents).unwrap(), Some(true));
    }

    #[test]
    fn parse_config_remove_fillers_rejects_invalid() {
        let contents = "remove_fillers = \"bogus\"\n";
        assert!(parse_config_remove_fillers(contents).is_err());

        let contents = "remove_fillers = 123\n";
        assert!(parse_config_remove_fillers(contents).is_err());
    }
}
