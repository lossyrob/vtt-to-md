//! Command-line interface argument parsing and validation.
//!
//! This module handles parsing command-line arguments using clap's derive macros,
//! validates argument combinations, and provides helpful error messages and usage text.

use crate::error::VttError;
use clap::{Parser, ValueEnum};
use std::path::{Path, PathBuf};

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

    /// Timestamp inclusion mode
    #[arg(
        long,
        value_name = "MODE",
        default_value = "none",
        help = "Timestamp inclusion mode: none, first (first cue of each speaker turn), or each (every cue)"
    )]
    pub include_timestamps: TimestampMode,
}

/// Timestamp inclusion mode for output
#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum TimestampMode {
    /// Don't include timestamps in output
    None,
    /// Include timestamp from first cue of each speaker turn
    First,
    /// Include timestamp for each original cue
    Each,
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
            self.output = Some(derive_output_path(&self.input));
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
fn derive_output_path(input: &Path) -> PathBuf {
    input.with_extension("md")
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
