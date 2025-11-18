//! Markdown generation and file output.
//!
//! This module handles formatting consolidated speaker segments into Markdown format
//! (bold speaker names followed by text) and writing the output to files or stdout.
//! It includes safeguards for file overwriting and proper permission handling.

use crate::cli::TimestampMode;
use crate::consolidator::SpeakerSegment;
use crate::error::VttError;
use std::fs;
use std::io::{self, Write};
use std::path::Path;

/// Format speaker segments as Markdown text.
///
/// Each segment is formatted as `**SpeakerName:** text` with double newlines
/// between segments. When timestamps are included, they're prepended as
/// `[HH:MM:SS.mmm] **SpeakerName:** text`.
///
/// # Arguments
///
/// * `segments` - The consolidated speaker segments to format
/// * `timestamp_mode` - How to include timestamps (None, First, or Each)
///
/// # Returns
///
/// A String containing the formatted Markdown content.
///
/// # Example
///
/// ```rust,ignore
/// let segments = vec![
///     SpeakerSegment {
///         speaker: "Alice".to_string(),
///         text: "Hello world.".to_string(),
///         timestamp: None,
///         timestamps: vec![],
///     },
/// ];
/// let markdown = format_markdown(&segments, TimestampMode::None);
/// // Result: "**Alice:** Hello world.\n\n"
/// ```
pub fn format_markdown(segments: &[SpeakerSegment], timestamp_mode: TimestampMode) -> String {
    let mut result = String::new();

    for segment in segments {
        match timestamp_mode {
            TimestampMode::None => {
                result.push_str(&format!("**{}:** {}\n\n", segment.speaker, segment.text));
            }
            TimestampMode::First => {
                if let Some(ref timestamp) = segment.timestamp {
                    result.push_str(&format!(
                        "[{}] **{}:** {}\n\n",
                        timestamp, segment.speaker, segment.text
                    ));
                } else {
                    result.push_str(&format!("**{}:** {}\n\n", segment.speaker, segment.text));
                }
            }
            TimestampMode::Each => {
                // TimestampMode::Each displays the first timestamp for each speaker segment
                // with the full consolidated text. This is a simplified implementation that
                // shows when the speaker turn began rather than splitting text by original
                // cue boundaries (which are lost during consolidation).
                // This aligns with the consolidator's text joining strategy.
                if !segment.timestamps.is_empty() {
                    result.push_str(&format!(
                        "[{}] **{}:** {}\n\n",
                        segment.timestamps[0], segment.speaker, segment.text
                    ));
                } else {
                    result.push_str(&format!("**{}:** {}\n\n", segment.speaker, segment.text));
                }
            }
        }
    }

    result
}

/// Write Markdown content to a file with appropriate safeguards.
///
/// This function checks if the output file exists and respects the
/// --force and --no-clobber flags. It handles permission errors and
/// other I/O errors appropriately.
///
/// # Arguments
///
/// * `content` - The Markdown content to write
/// * `output_path` - The path to write to
/// * `force` - Whether to overwrite existing files
/// * `no_clobber` - Whether to skip if file exists
///
/// # Returns
///
/// Returns `Ok(())` if successful, or `Err(VttError)` if:
/// - File exists and --force not set (OutputExists)
/// - Permission denied (PermissionDenied)
/// - Other I/O errors (WriteError)
///
/// # Example
///
/// ```rust,ignore
/// write_markdown_file("**Alice:** Hello", Path::new("output.md"), false, false)?;
/// ```
pub fn write_markdown_file(
    content: &str,
    output_path: &Path,
    force: bool,
    no_clobber: bool,
) -> Result<(), VttError> {
    // Check if output file exists
    if output_path.exists() {
        if no_clobber {
            // Skip silently (this is success case for --no-clobber)
            return Ok(());
        }
        if !force {
            return Err(VttError::OutputExists {
                path: output_path.to_path_buf(),
            });
        }
        // If force is true, we'll overwrite
    }

    // Write the file
    fs::write(output_path, content).map_err(|e| {
        if e.kind() == io::ErrorKind::PermissionDenied {
            VttError::PermissionDenied {
                path: output_path.to_path_buf(),
            }
        } else {
            VttError::WriteError {
                path: output_path.to_path_buf(),
                source: e,
            }
        }
    })?;

    Ok(())
}

/// Write Markdown content to stdout.
///
/// # Arguments
///
/// * `content` - The Markdown content to print
///
/// # Returns
///
/// Returns `Ok(())` if successful, or `Err(VttError)` for I/O errors.
pub fn write_markdown_stdout(content: &str) -> Result<(), VttError> {
    io::stdout()
        .write_all(content.as_bytes())
        .map_err(VttError::IoError)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_format_markdown_no_timestamps() {
        let segments = vec![
            SpeakerSegment {
                speaker: "Alice".to_string(),
                text: "Hello world.".to_string(),
                timestamp: None,
                timestamps: vec![],
            },
            SpeakerSegment {
                speaker: "Bob".to_string(),
                text: "Hi Alice!".to_string(),
                timestamp: None,
                timestamps: vec![],
            },
        ];

        let markdown = format_markdown(&segments, TimestampMode::None);

        assert_eq!(
            markdown,
            "**Alice:** Hello world.\n\n**Bob:** Hi Alice!\n\n"
        );
    }

    #[test]
    fn test_format_markdown_first_timestamp() {
        let segments = vec![
            SpeakerSegment {
                speaker: "Alice".to_string(),
                text: "Hello world.".to_string(),
                timestamp: Some("00:00:01.000".to_string()),
                timestamps: vec![],
            },
            SpeakerSegment {
                speaker: "Bob".to_string(),
                text: "Hi Alice!".to_string(),
                timestamp: Some("00:00:05.000".to_string()),
                timestamps: vec![],
            },
        ];

        let markdown = format_markdown(&segments, TimestampMode::First);

        assert_eq!(
            markdown,
            "[00:00:01.000] **Alice:** Hello world.\n\n[00:00:05.000] **Bob:** Hi Alice!\n\n"
        );
    }

    #[test]
    fn test_format_markdown_each_timestamp() {
        let segments = vec![SpeakerSegment {
            speaker: "Alice".to_string(),
            text: "Hello world. How are you?".to_string(),
            timestamp: None,
            timestamps: vec!["00:00:01.000".to_string(), "00:00:02.000".to_string()],
        }];

        let markdown = format_markdown(&segments, TimestampMode::Each);

        // For now, Each mode shows first timestamp with full text
        assert_eq!(
            markdown,
            "[00:00:01.000] **Alice:** Hello world. How are you?\n\n"
        );
    }

    #[test]
    fn test_write_markdown_file_success() {
        let temp_file = std::env::temp_dir().join("test_write_success.md");
        let content = "**Alice:** Hello world.\n\n";

        // Clean up any existing file
        fs::remove_file(&temp_file).ok();

        let result = write_markdown_file(content, &temp_file, false, false);
        assert!(result.is_ok());

        // Verify content
        let written = fs::read_to_string(&temp_file).unwrap();
        assert_eq!(written, content);

        // Clean up
        fs::remove_file(&temp_file).ok();
    }

    #[test]
    fn test_write_markdown_file_exists_no_force() {
        let temp_file = std::env::temp_dir().join("test_write_exists.md");

        // Create existing file
        fs::write(&temp_file, "existing content").unwrap();

        let result = write_markdown_file("new content", &temp_file, false, false);

        assert!(result.is_err());
        match result {
            Err(VttError::OutputExists { .. }) => {}
            _ => panic!("Expected OutputExists error"),
        }

        // Clean up
        fs::remove_file(&temp_file).ok();
    }

    #[test]
    fn test_write_markdown_file_exists_with_force() {
        let temp_file = std::env::temp_dir().join("test_write_force.md");

        // Create existing file
        fs::write(&temp_file, "existing content").unwrap();

        let result = write_markdown_file("new content", &temp_file, true, false);
        assert!(result.is_ok());

        // Verify content was overwritten
        let written = fs::read_to_string(&temp_file).unwrap();
        assert_eq!(written, "new content");

        // Clean up
        fs::remove_file(&temp_file).ok();
    }

    #[test]
    fn test_write_markdown_file_no_clobber() {
        let temp_file = std::env::temp_dir().join("test_write_no_clobber.md");

        // Create existing file
        fs::write(&temp_file, "existing content").unwrap();

        let result = write_markdown_file("new content", &temp_file, false, true);
        assert!(result.is_ok()); // Should succeed but not write

        // Verify content was NOT overwritten
        let written = fs::read_to_string(&temp_file).unwrap();
        assert_eq!(written, "existing content");

        // Clean up
        fs::remove_file(&temp_file).ok();
    }
}
