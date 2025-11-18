//! WebVTT file parsing and cue extraction.
//!
//! This module provides functionality to parse VTT files, extract speaker attributions
//! from voice tags, and handle platform-specific variations (Teams, Zoom, Google Meet).
//! It includes text sanitization, HTML entity decoding, and robust error handling for
//! malformed VTT content.

use crate::error::VttError;
use regex::Regex;
use std::fs;
use std::io::{self, BufRead, BufReader};
use std::path::Path;
use unicode_normalization::UnicodeNormalization;

/// Represents a single VTT cue with optional timestamp, speaker, and text content.
#[derive(Debug, Clone, PartialEq)]
pub struct Cue {
    /// Optional timestamp for when this cue appears (format: HH:MM:SS.mmm)
    pub timestamp: Option<String>,
    /// Optional speaker name (extracted from <v> tags)
    pub speaker: Option<String>,
    /// The text content of the cue
    pub text: String,
}

/// Represents a parsed VTT document containing a collection of cues.
#[derive(Debug, Clone, PartialEq)]
pub struct VttDocument {
    /// The collection of cues extracted from the VTT file
    pub cues: Vec<Cue>,
    /// Whether this VTT file contains voice tags (Teams-style format)
    pub has_voice_tags: bool,
}

impl VttDocument {
    /// Parse a VTT file from the given path.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the VTT file to parse
    ///
    /// # Returns
    ///
    /// Returns `Ok(VttDocument)` if parsing succeeds, or `Err(VttError)` if:
    /// - File cannot be read (not found, permission denied, etc.)
    /// - File is not a valid VTT file (missing WEBVTT header)
    /// - File contains malformed content that cannot be parsed
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let doc = VttDocument::parse("transcript.vtt")?;
    /// for cue in doc.cues {
    ///     println!("{:?}: {}", cue.speaker, cue.text);
    /// }
    /// ```
    pub fn parse<P: AsRef<Path>>(path: P) -> Result<Self, VttError> {
        let path = path.as_ref();

        // Open and read the file
        let file = fs::File::open(path).map_err(|e| {
            if e.kind() == io::ErrorKind::NotFound {
                VttError::FileNotFound {
                    path: path.to_path_buf(),
                }
            } else if e.kind() == io::ErrorKind::PermissionDenied {
                VttError::PermissionDenied {
                    path: path.to_path_buf(),
                }
            } else {
                VttError::IoError(e)
            }
        })?;

        let reader = BufReader::new(file);
        let mut lines = reader.lines();

        // Validate WEBVTT header
        let first_line = lines
            .next()
            .ok_or_else(|| VttError::ParseError {
                reason: "Empty file".to_string(),
            })?
            .map_err(VttError::IoError)?;

        if !first_line.trim().starts_with("WEBVTT") {
            return Err(VttError::ParseError {
                reason: "Missing WEBVTT header".to_string(),
            });
        }

        // Parse cues from the remaining lines
        let (cues, has_voice_tags) = parse_cues(lines)?;

        Ok(VttDocument { cues, has_voice_tags })
    }
}

/// Parse cues from VTT file lines.
/// Returns the list of cues and a boolean indicating if any voice tags were found.
fn parse_cues<I>(lines: I) -> Result<(Vec<Cue>, bool), VttError>
where
    I: Iterator<Item = io::Result<String>>,
{
    let timestamp_regex =
        Regex::new(r"^\s*(\d{2}:\d{2}:\d{2}\.\d{3})\s*-->\s*(\d{2}:\d{2}:\d{2}\.\d{3})").unwrap();
    let mut cues = Vec::new();
    let mut current_timestamp: Option<String> = None;
    let mut current_text = Vec::new();
    let mut in_metadata_block = false;

    for line_result in lines {
        let line = line_result.map_err(VttError::IoError)?;
        let trimmed = line.trim();

        // Skip metadata blocks (NOTE, STYLE, REGION)
        if trimmed.starts_with("NOTE")
            || trimmed.starts_with("STYLE")
            || trimmed.starts_with("REGION")
        {
            in_metadata_block = true;
            continue;
        }

        // Check if this is a timestamp line
        if let Some(captures) = timestamp_regex.captures(&line) {
            // Save any previous cue text
            if !current_text.is_empty() {
                save_cue(&mut cues, current_timestamp.clone(), &current_text)?;
                current_text.clear();
            }

            // Start new cue with timestamp
            current_timestamp = Some(captures[1].to_string());
            in_metadata_block = false;
            continue;
        }

        // Empty line: end of cue or metadata block
        if trimmed.is_empty() {
            if !current_text.is_empty() {
                save_cue(&mut cues, current_timestamp.clone(), &current_text)?;
                current_text.clear();
                current_timestamp = None;
            }
            in_metadata_block = false;
            continue;
        }

        // Skip lines in metadata blocks
        if in_metadata_block {
            continue;
        }

        // Skip cue identifiers (lines before timestamp that are just numbers/IDs)
        if current_timestamp.is_none() && !current_text.is_empty() {
            continue;
        }

        // Collect cue text
        if current_timestamp.is_some() {
            current_text.push(line);
        }
    }

    // Save final cue if any
    if !current_text.is_empty() {
        save_cue(&mut cues, current_timestamp, &current_text)?;
    }

    // Check if any cues have speakers (indicating voice tags were present)
    let has_voice_tags = cues.iter().any(|cue| cue.speaker.is_some());

    // Sort cues by timestamp to handle out-of-order cues in VTT files
    // (some formats like Teams can have interleaved cues)
    cues.sort_by(|a, b| {
        match (&a.timestamp, &b.timestamp) {
            (Some(ts_a), Some(ts_b)) => ts_a.cmp(ts_b),
            (Some(_), None) => std::cmp::Ordering::Less,
            (None, Some(_)) => std::cmp::Ordering::Greater,
            (None, None) => std::cmp::Ordering::Equal,
        }
    });

    Ok((cues, has_voice_tags))
}

/// Save a cue by extracting speaker and cleaning text.
fn save_cue(
    cues: &mut Vec<Cue>,
    timestamp: Option<String>,
    text_lines: &[String],
) -> Result<(), VttError> {
    // Join lines and extract speaker
    let combined = text_lines.join("\n");
    let (speaker, text) = extract_speaker_and_text(&combined);

    // Clean the text
    let cleaned_text = clean_text(&text);

    // Skip empty cues
    if cleaned_text.trim().is_empty() {
        return Ok(());
    }

    // Sanitize speaker name if present
    let sanitized_speaker = speaker.and_then(|s| sanitize_speaker_name(&s));

    cues.push(Cue {
        timestamp,
        speaker: sanitized_speaker,
        text: cleaned_text,
    });

    Ok(())
}

/// Extract speaker name from <v> voice tags and return speaker and remaining text.
fn extract_speaker_and_text(text: &str) -> (Option<String>, String) {
    // First try to match closed voice tags: <v Speaker>text</v>
    // Use (?s) flag to make . match newlines (for multi-line cue content)
    let voice_closed_regex = Regex::new(r"(?s)<v\s+([^>]*)>(.*?)</v>").unwrap();

    if let Some(captures) = voice_closed_regex.captures(text) {
        let speaker_str = captures.get(1).map(|m| m.as_str().trim()).unwrap_or("");
        let speaker = if speaker_str.is_empty() {
            None
        } else {
            Some(speaker_str.to_string())
        };
        let content = captures.get(2).map(|m| m.as_str()).unwrap_or("");

        return (speaker, content.to_string());
    }

    // Also try to match <v>text</v> (empty speaker)
    // Use (?s) flag to make . match newlines
    let voice_empty_regex = Regex::new(r"(?s)<v>(.*?)</v>").unwrap();
    if let Some(captures) = voice_empty_regex.captures(text) {
        let content = captures.get(1).map(|m| m.as_str()).unwrap_or("");
        return (None, content.to_string());
    }

    // Check for voice tag without closing tag: <v Speaker>text
    // Use (?s) flag to make . match newlines
    let voice_open_regex = Regex::new(r"(?s)<v\s+([^>]*)>(.*)").unwrap();
    if let Some(captures) = voice_open_regex.captures(text) {
        let speaker_str = captures.get(1).map(|m| m.as_str()).unwrap_or("").trim();
        let speaker = if speaker_str.is_empty() {
            None
        } else {
            Some(speaker_str.to_string())
        };
        let content = captures.get(2).map(|m| m.as_str()).unwrap_or("");
        return (speaker, content.to_string());
    }

    // No voice tag found
    (None, text.to_string())
}

/// Clean text by stripping HTML tags, decoding entities, and normalizing whitespace.
fn clean_text(text: &str) -> String {
    // Strip HTML tags (except voice tags which should already be processed)
    let html_tag_regex = Regex::new(r"<[^>]+>").unwrap();
    let text = html_tag_regex.replace_all(text, "");

    // Decode HTML entities
    let text = decode_html_entities(&text);

    // Normalize whitespace: collapse multiple spaces, normalize newlines
    let whitespace_regex = Regex::new(r"\s+").unwrap();
    let text = whitespace_regex.replace_all(&text, " ");

    text.trim().to_string()
}

/// Decode common HTML character references.
fn decode_html_entities(text: &str) -> String {
    text.replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
        .replace("&#x27;", "'")
}

/// Sanitize speaker name: remove @ symbols, apply NFC normalization,
/// escape Markdown special chars, and return None for whitespace-only names.
fn sanitize_speaker_name(name: &str) -> Option<String> {
    // Remove @ symbols (Teams anonymized users)
    let name = name.replace('@', "");

    // Apply Unicode NFC normalization
    let name: String = name.nfc().collect();

    // Trim whitespace
    let name = name.trim();

    // Return None for empty or whitespace-only names
    if name.is_empty() {
        return None;
    }

    // Escape Markdown special characters
    let name = escape_markdown(name);

    Some(name)
}

/// Escape Markdown special characters in text.
fn escape_markdown(text: &str) -> String {
    let mut result = String::with_capacity(text.len());

    for ch in text.chars() {
        match ch {
            '*' | '_' | '#' | '[' | ']' | '(' | ')' | '{' | '}' | '!' | '>' | '|' | '`' | '\\' => {
                result.push('\\');
                result.push(ch);
            }
            _ => result.push(ch),
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_sanitize_speaker_name() {
        // Test @ symbol removal
        assert_eq!(
            sanitize_speaker_name("@anonymous"),
            Some("anonymous".to_string())
        );

        // Test markdown escaping
        assert_eq!(
            sanitize_speaker_name("John*Doe"),
            Some("John\\*Doe".to_string())
        );

        // Test whitespace-only returns None
        assert_eq!(sanitize_speaker_name("   "), None);
        assert_eq!(sanitize_speaker_name(""), None);

        // Test normal name
        assert_eq!(
            sanitize_speaker_name("John Doe"),
            Some("John Doe".to_string())
        );
    }

    #[test]
    fn test_decode_html_entities() {
        assert_eq!(decode_html_entities("&amp;"), "&");
        assert_eq!(decode_html_entities("&lt;"), "<");
        assert_eq!(decode_html_entities("&gt;"), ">");
        assert_eq!(decode_html_entities("&quot;"), "\"");
        assert_eq!(decode_html_entities("&#39;"), "'");
        assert_eq!(decode_html_entities("A &amp; B"), "A & B");
    }

    #[test]
    fn test_extract_speaker_and_text() {
        // Test with voice tags
        let (speaker, text) = extract_speaker_and_text("<v John Doe>Hello world</v>");
        assert_eq!(speaker, Some("John Doe".to_string()));
        assert_eq!(text, "Hello world");

        // Test without voice tags
        let (speaker, text) = extract_speaker_and_text("Hello world");
        assert_eq!(speaker, None);
        assert_eq!(text, "Hello world");

        // Test with empty speaker (voice tag without speaker name)
        let (speaker, text) = extract_speaker_and_text("<v>Hello world</v>");
        assert_eq!(speaker, None);
        assert_eq!(text, "Hello world");

        // Test with unclosed voice tag
        let (speaker, text) = extract_speaker_and_text("<v Jane Smith>Hello world");
        assert_eq!(speaker, Some("Jane Smith".to_string()));
        assert_eq!(text, "Hello world");

        // Test with multi-line voice tag (newlines in content)
        let (speaker, text) = extract_speaker_and_text("<v Alice>Line one\nLine two</v>");
        assert_eq!(speaker, Some("Alice".to_string()));
        assert_eq!(text, "Line one\nLine two");

        // Test with multi-line voice tag with empty speaker
        let (speaker, text) = extract_speaker_and_text("<v>First line\nSecond line</v>");
        assert_eq!(speaker, None);
        assert_eq!(text, "First line\nSecond line");
    }

    #[test]
    fn test_clean_text() {
        // Test HTML tag stripping
        assert_eq!(clean_text("<b>Bold</b> text"), "Bold text");

        // Test entity decoding
        assert_eq!(clean_text("A &amp; B"), "A & B");

        // Test whitespace normalization
        assert_eq!(clean_text("Hello   world"), "Hello world");
        assert_eq!(clean_text("Hello\n\nworld"), "Hello world");
    }

    #[test]
    fn test_escape_markdown() {
        assert_eq!(escape_markdown("Normal text"), "Normal text");
        assert_eq!(escape_markdown("*bold*"), "\\*bold\\*");
        assert_eq!(escape_markdown("_italic_"), "\\_italic\\_");
        assert_eq!(escape_markdown("[link]"), "\\[link\\]");
        assert_eq!(escape_markdown("# heading"), "\\# heading");
    }

    #[test]
    fn test_parse_valid_vtt_with_speakers() {
        let vtt_content = r#"WEBVTT

1
00:00:01.000 --> 00:00:03.000
<v Alice>Hello, this is Alice speaking.</v>

2
00:00:04.000 --> 00:00:06.000
<v Bob>Hi Alice, this is Bob.</v>

3
00:00:07.000 --> 00:00:09.000
<v Alice>How are you today?</v>
"#;

        let temp_file = std::env::temp_dir().join("test_parse_valid.vtt");
        fs::write(&temp_file, vtt_content).unwrap();

        let result = VttDocument::parse(&temp_file);
        assert!(result.is_ok());

        let doc = result.unwrap();
        assert_eq!(doc.cues.len(), 3);

        assert_eq!(doc.cues[0].speaker, Some("Alice".to_string()));
        assert_eq!(doc.cues[0].text, "Hello, this is Alice speaking.");
        assert_eq!(doc.cues[0].timestamp, Some("00:00:01.000".to_string()));

        assert_eq!(doc.cues[1].speaker, Some("Bob".to_string()));
        assert_eq!(doc.cues[1].text, "Hi Alice, this is Bob.");

        assert_eq!(doc.cues[2].speaker, Some("Alice".to_string()));
        assert_eq!(doc.cues[2].text, "How are you today?");

        fs::remove_file(&temp_file).ok();
    }

    #[test]
    fn test_parse_missing_webvtt_header() {
        let vtt_content = r#"This is not a VTT file

00:00:01.000 --> 00:00:03.000
Some text
"#;

        let temp_file = std::env::temp_dir().join("test_missing_header.vtt");
        fs::write(&temp_file, vtt_content).unwrap();

        let result = VttDocument::parse(&temp_file);
        assert!(result.is_err());

        match result {
            Err(VttError::ParseError { reason }) => {
                assert!(reason.contains("WEBVTT"));
            }
            _ => panic!("Expected ParseError"),
        }

        fs::remove_file(&temp_file).ok();
    }

    #[test]
    fn test_parse_file_not_found() {
        let result = VttDocument::parse("nonexistent_file.vtt");
        assert!(result.is_err());

        match result {
            Err(VttError::FileNotFound { .. }) => {}
            _ => panic!("Expected FileNotFound error"),
        }
    }

    #[test]
    fn test_parse_without_speaker_tags() {
        let vtt_content = r#"WEBVTT

1
00:00:01.000 --> 00:00:03.000
This text has no speaker tag.

2
00:00:04.000 --> 00:00:06.000
Neither does this one.
"#;

        let temp_file = std::env::temp_dir().join("test_no_speakers.vtt");
        fs::write(&temp_file, vtt_content).unwrap();

        let result = VttDocument::parse(&temp_file);
        assert!(result.is_ok());

        let doc = result.unwrap();
        assert_eq!(doc.cues.len(), 2);

        assert_eq!(doc.cues[0].speaker, None);
        assert_eq!(doc.cues[0].text, "This text has no speaker tag.");

        assert_eq!(doc.cues[1].speaker, None);
        assert_eq!(doc.cues[1].text, "Neither does this one.");

        fs::remove_file(&temp_file).ok();
    }

    #[test]
    fn test_parse_with_html_entities() {
        let vtt_content = r#"WEBVTT

1
00:00:01.000 --> 00:00:03.000
<v Alice>This &amp; that are &lt;important&gt;.</v>
"#;

        let temp_file = std::env::temp_dir().join("test_entities.vtt");
        fs::write(&temp_file, vtt_content).unwrap();

        let result = VttDocument::parse(&temp_file);
        assert!(result.is_ok());

        let doc = result.unwrap();
        assert_eq!(doc.cues.len(), 1);
        assert_eq!(doc.cues[0].text, "This & that are <important>.");

        fs::remove_file(&temp_file).ok();
    }

    #[test]
    fn test_parse_with_at_symbol_speaker() {
        let vtt_content = r#"WEBVTT

1
00:00:01.000 --> 00:00:03.000
<v @anonymous>Hello from anonymous user.</v>
"#;

        let temp_file = std::env::temp_dir().join("test_at_symbol.vtt");
        fs::write(&temp_file, vtt_content).unwrap();

        let result = VttDocument::parse(&temp_file);
        assert!(result.is_ok());

        let doc = result.unwrap();
        assert_eq!(doc.cues.len(), 1);
        // @ should be removed
        assert_eq!(doc.cues[0].speaker, Some("anonymous".to_string()));

        fs::remove_file(&temp_file).ok();
    }

    #[test]
    fn test_parse_with_metadata_blocks() {
        let vtt_content = r#"WEBVTT

NOTE This is a comment

STYLE
::cue {
  color: yellow;
}

1
00:00:01.000 --> 00:00:03.000
<v Alice>This should be parsed.</v>

NOTE Another comment

2
00:00:04.000 --> 00:00:06.000
<v Bob>This should also be parsed.</v>
"#;

        let temp_file = std::env::temp_dir().join("test_metadata.vtt");
        fs::write(&temp_file, vtt_content).unwrap();

        let result = VttDocument::parse(&temp_file);
        assert!(result.is_ok());

        let doc = result.unwrap();
        assert_eq!(doc.cues.len(), 2);
        assert_eq!(doc.cues[0].speaker, Some("Alice".to_string()));
        assert_eq!(doc.cues[1].speaker, Some("Bob".to_string()));

        fs::remove_file(&temp_file).ok();
    }

    #[test]
    fn test_parse_empty_file() {
        let temp_file = std::env::temp_dir().join("test_empty.vtt");
        fs::write(&temp_file, "").unwrap();

        let result = VttDocument::parse(&temp_file);
        assert!(result.is_err());

        match result {
            Err(VttError::ParseError { reason }) => {
                assert!(reason.contains("Empty"));
            }
            _ => panic!("Expected ParseError for empty file"),
        }

        fs::remove_file(&temp_file).ok();
    }

    #[test]
    fn test_parse_whitespace_only_speaker() {
        let vtt_content = r#"WEBVTT

1
00:00:01.000 --> 00:00:03.000
<v   >Text with whitespace-only speaker.</v>
"#;

        let temp_file = std::env::temp_dir().join("test_whitespace_speaker.vtt");
        fs::write(&temp_file, vtt_content).unwrap();

        let result = VttDocument::parse(&temp_file);
        assert!(result.is_ok());

        let doc = result.unwrap();
        assert_eq!(doc.cues.len(), 1);
        // Whitespace-only speaker should be None
        assert_eq!(doc.cues[0].speaker, None);
        assert_eq!(doc.cues[0].text, "Text with whitespace-only speaker.");

        fs::remove_file(&temp_file).ok();
    }
}
