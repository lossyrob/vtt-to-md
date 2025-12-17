use crate::cli::TimestampMode;
use std::fs;
use std::path::PathBuf;
use std::str::FromStr;

const ENV_INCLUDE_TIMESTAMPS: &str = "VTT_TO_MD_INCLUDE_TIMESTAMPS";
const APP_DIR_NAME: &str = "vtt-to-md";
const CONFIG_FILE_NAME: &str = "config.toml";
const CONFIG_KEY_INCLUDE_TIMESTAMPS: &str = "include_timestamps";

pub fn resolve_default_timestamp_mode() -> (Option<TimestampMode>, Vec<String>) {
    let mut warnings = Vec::new();

    // 1) Environment variable (highest non-CLI precedence)
    if let Ok(raw_value) = std::env::var(ENV_INCLUDE_TIMESTAMPS) {
        let value = raw_value.trim();
        if value.is_empty() {
            warnings.push(format!(
                "{ENV_INCLUDE_TIMESTAMPS} is set but empty; ignoring"
            ));
        } else {
            match TimestampMode::from_str(value) {
                Ok(mode) => return (Some(mode), warnings),
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
        Ok(mode) => (mode, warnings),
        Err(err) => {
            warnings.push(format!(
                "invalid config file '{}': {err}; ignoring",
                config_path.display()
            ));
            (None, warnings)
        }
    }
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

fn parse_config_include_timestamps(contents: &str) -> Result<Option<TimestampMode>, String> {
    let value: toml::Value = contents
        .parse()
        .map_err(|err| format!("TOML parse error: {err}"))?;

    let Some(include_value) = value.get(CONFIG_KEY_INCLUDE_TIMESTAMPS) else {
        return Ok(None);
    };

    let Some(include_str) = include_value.as_str() else {
        return Err(format!(
            "'{CONFIG_KEY_INCLUDE_TIMESTAMPS}' must be a string"
        ));
    };

    let parsed = TimestampMode::from_str(include_str)
        .map_err(|err| format!("invalid {CONFIG_KEY_INCLUDE_TIMESTAMPS}='{include_str}': {err}"))?;

    Ok(Some(parsed))
}

#[cfg(test)]
mod tests {
    use super::parse_config_include_timestamps;
    use crate::cli::TimestampMode;

    #[test]
    fn parse_config_include_timestamps_missing_key_returns_none() {
        let contents = "other = 'x'\n";
        assert_eq!(parse_config_include_timestamps(contents).unwrap(), None);
    }

    #[test]
    fn parse_config_include_timestamps_accepts_valid_values() {
        let contents = "include_timestamps = \"first\"\n";
        assert_eq!(
            parse_config_include_timestamps(contents).unwrap(),
            Some(TimestampMode::First)
        );
    }

    #[test]
    fn parse_config_include_timestamps_rejects_invalid_value() {
        let contents = "include_timestamps = \"bogus\"\n";
        assert!(parse_config_include_timestamps(contents).is_err());
    }

    #[test]
    fn parse_config_include_timestamps_rejects_non_string() {
        let contents = "include_timestamps = 1\n";
        assert!(parse_config_include_timestamps(contents).is_err());
    }
}
