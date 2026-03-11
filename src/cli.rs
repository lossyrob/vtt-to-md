//! Command-line interface argument parsing and validation.
//!
//! This module handles parsing command-line arguments using clap's derive macros,
//! validates argument combinations, and provides helpful error messages and usage text.

use crate::error::VttError;
use clap::Parser;
use std::path::{Path, PathBuf};
use std::str::FromStr;

/// VTT to Markdown converter - Convert WebVTT transcript files to readable Markdown
#[derive(Parser, Debug)]
#[command(
    name = "vtt-to-md",
    version,
    about = "Convert WebVTT transcript files to readable Markdown",
    long_about = "Converts WebVTT (Web Video Text Tracks) transcript files from meeting platforms\n\
                  (Microsoft Teams, Zoom, Google Meet) to readable Markdown format with bold\n\
                  speaker names and consolidated text paragraphs."
)]
pub struct Args {
    /// Path to the input VTT file
    #[arg(value_name = "INPUT", help = "Path to the input VTT file")]
    pub input: PathBuf,

    /// Path to the output Markdown file (defaults to INPUT with .md extension)
    #[arg(value_name = "OUTPUT", help = "Path to the output Markdown file")]
    pub output: Option<PathBuf>,

    /// Overwrite existing output file
    #[arg(
        short,
        long,
        conflicts_with = "no_clobber",
        help = "Overwrite existing output file"
    )]
    pub force: bool,

    /// Skip conversion if output file exists
    #[arg(
        short = 'n',
        long,
        conflicts_with = "force",
        help = "Skip conversion if output file exists"
    )]
    pub no_clobber: bool,

    /// Print Markdown to stdout instead of writing to file
    #[arg(long, help = "Print Markdown to stdout instead of writing to file")]
    pub stdout: bool,

    /// Custom label for cues without speaker attribution
    #[arg(
        long,
        value_name = "LABEL",
        default_value = "Unknown",
        help = "Custom label for cues without speaker attribution"
    )]
    pub unknown_speaker: String,

    /// Filter out cues without speaker attribution
    #[arg(
        long,
        help = "Filter out cues without speaker attribution (removes 'Unknown' speaker segments).\n\
                Auto-enabled for Teams-style VTT files with <v> tags unless explicitly disabled with --no-filter-unknown"
    )]
    pub filter_unknown: bool,

    /// Disable automatic filtering of unknown speakers for Teams-style VTT files
    #[arg(
        long,
        conflicts_with = "filter_unknown",
        help = "Disable automatic filtering of unknown speakers for Teams-style VTT files"
    )]
    pub no_filter_unknown: bool,

    /// Remove filler words and speech disfluencies from the transcript
    #[arg(
        long,
        help = "Remove filler words and speech disfluencies (uh, um, you know, etc.)"
    )]
    pub remove_fillers: bool,

    /// Disable auto-increment of output filename on collision
    #[arg(
        long,
        help = "Disable auto-increment of output filename when file exists (use with --force to overwrite)"
    )]
    pub no_auto_increment: bool,

    /// Include timestamps (one per consolidated speaker turn)
    #[arg(
        long,
        value_name = "BOOL",
        num_args = 0..=1,
        default_missing_value = "true",
        help = "Include timestamps in output (one per consolidated speaker turn).\n\
                Use --include-timestamps or --include-timestamps=false.\n\
                Legacy values (none|first|each) are accepted with a deprecation warning."
    )]
    pub include_timestamps: Option<IncludeTimestampsSetting>,
}

/// Parsed value for timestamp inclusion.
///
/// This is a boolean setting, but we accept legacy values (`none|first|each`) for backward
/// compatibility and surface them as `is_legacy=true` so callers can emit deprecation warnings.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct IncludeTimestampsSetting {
    pub value: bool,
    pub is_legacy: bool,
}

impl IncludeTimestampsSetting {
    fn parse_bool(value: &str) -> Option<bool> {
        match value.trim().to_ascii_lowercase().as_str() {
            "true" | "1" | "yes" | "y" | "on" => Some(true),
            "false" | "0" | "no" | "n" | "off" => Some(false),
            _ => None,
        }
    }
}

impl FromStr for IncludeTimestampsSetting {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let trimmed = s.trim();
        if trimmed.is_empty() {
            return Err("include_timestamps value is empty".to_string());
        }

        if let Some(value) = Self::parse_bool(trimmed) {
            return Ok(Self {
                value,
                is_legacy: false,
            });
        }

        if trimmed.eq_ignore_ascii_case("none") {
            return Ok(Self {
                value: false,
                is_legacy: true,
            });
        }
        if trimmed.eq_ignore_ascii_case("first") || trimmed.eq_ignore_ascii_case("each") {
            return Ok(Self {
                value: true,
                is_legacy: true,
            });
        }

        Err(format!(
            "invalid include_timestamps value '{trimmed}' (expected a boolean like true/false, or legacy none/first/each)"
        ))
    }
}

impl Args {
    /// Validate arguments and derive output path if not specified.
    ///
    /// This method checks for invalid argument combinations and derives
    /// the output path from the input path if not explicitly provided.
    ///
    /// # Errors
    ///
    /// Returns `VttError::UsageError` if:
    /// - Input and output paths are the same
    /// - Other validation constraints are violated
    pub fn validate(&mut self) -> Result<(), VttError> {
        // Derive output path if not specified and not using stdout
        if self.output.is_none() && !self.stdout {
            if self.no_auto_increment {
                // Old behavior: simple extension replacement
                self.output = Some(self.input.with_extension("md"));
            } else {
                // New default: auto-increment on collision
                self.output = Some(derive_output_path(&self.input));
            }
        }

        // Check if input and output are the same file
        if let Some(ref output) = self.output
            && paths_equal(&self.input, output)
        {
            return Err(VttError::SameFile {
                path: self.input.clone(),
            });
        }

        Ok(())
    }

    /// Get the output path, returning None if stdout mode is enabled.
    pub fn get_output_path(&self) -> Option<&Path> {
        if self.stdout {
            None
        } else {
            self.output.as_deref()
        }
    }
}

/// Derive output path from input path by replacing extension with .md
/// and finding next available filename if collision occurs.
fn derive_output_path(input: &Path) -> PathBuf {
    let base_output = input.with_extension("md");
    find_available_path(&base_output)
}

/// Find next available path by adding (N) suffix if file exists.
///
/// Given path/to/file.md, tries:
/// - path/to/file.md
/// - path/to/file (1).md
/// - path/to/file (2).md
///
/// etc.
fn find_available_path(base_path: &Path) -> PathBuf {
    if !base_path.exists() {
        return base_path.to_path_buf();
    }

    let parent = base_path.parent().unwrap_or_else(|| Path::new(""));
    let extension = base_path.extension().and_then(|s| s.to_str()).unwrap_or("");
    let stem = base_path.file_stem().and_then(|s| s.to_str()).unwrap_or("");

    for i in 1..=9999 {
        let new_name = if extension.is_empty() {
            format!("{} ({})", stem, i)
        } else {
            format!("{} ({}).{}", stem, i, extension)
        };
        let new_path = parent.join(new_name);
        if !new_path.exists() {
            return new_path;
        }
    }

    // Fallback: if we somehow exhaust 9999 attempts, return the base path
    // (will likely fail during write, but this is an extreme edge case)
    base_path.to_path_buf()
}

/// Check if two paths refer to the same file.
///
/// This performs a simple comparison after canonicalization attempt.
/// If canonicalization fails (e.g., file doesn't exist yet), falls back
/// to path comparison.
fn paths_equal(path1: &Path, path2: &Path) -> bool {
    // Try to canonicalize both paths
    match (path1.canonicalize(), path2.canonicalize()) {
        (Ok(canon1), Ok(canon2)) => canon1 == canon2,
        _ => {
            // Fall back to direct comparison if canonicalization fails
            path1 == path2
        }
    }
}

#[cfg(test)]
mod tests {
    use super::IncludeTimestampsSetting;
    use std::str::FromStr;

    #[test]
    fn include_timestamps_setting_from_str_accepts_booleans_and_legacy_values() {
        assert_eq!(
            IncludeTimestampsSetting::from_str("true").unwrap(),
            IncludeTimestampsSetting {
                value: true,
                is_legacy: false
            }
        );
        assert_eq!(
            IncludeTimestampsSetting::from_str(" false ").unwrap(),
            IncludeTimestampsSetting {
                value: false,
                is_legacy: false
            }
        );
        assert_eq!(
            IncludeTimestampsSetting::from_str("ON").unwrap(),
            IncludeTimestampsSetting {
                value: true,
                is_legacy: false
            }
        );
        assert_eq!(
            IncludeTimestampsSetting::from_str("0").unwrap(),
            IncludeTimestampsSetting {
                value: false,
                is_legacy: false
            }
        );

        assert_eq!(
            IncludeTimestampsSetting::from_str("none").unwrap(),
            IncludeTimestampsSetting {
                value: false,
                is_legacy: true
            }
        );
        assert_eq!(
            IncludeTimestampsSetting::from_str("FIRST").unwrap(),
            IncludeTimestampsSetting {
                value: true,
                is_legacy: true
            }
        );
        assert_eq!(
            IncludeTimestampsSetting::from_str(" EaCh ").unwrap(),
            IncludeTimestampsSetting {
                value: true,
                is_legacy: true
            }
        );
    }

    #[test]
    fn include_timestamps_setting_from_str_rejects_invalid_values() {
        assert!(IncludeTimestampsSetting::from_str("").is_err());
        assert!(IncludeTimestampsSetting::from_str("bogus").is_err());
        assert!(IncludeTimestampsSetting::from_str("none-ish").is_err());
    }
}
