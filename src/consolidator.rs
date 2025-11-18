//! Speaker segment consolidation logic.
//!
//! This module implements the logic to merge consecutive cues from the same speaker
//! into coherent paragraphs. It handles speaker changes, applies unknown speaker labels,
//! and joins text intelligently while respecting sentence boundaries.
//!
//! # Example
//!
//! ```rust,ignore
//! use vtt_to_md::consolidator::consolidate_cues;
//! use vtt_to_md::parser::Cue;
//! use vtt_to_md::cli::TimestampMode;
//!
//! let cues = vec![
//!     Cue { speaker: Some("Alice".to_string()), text: "Hello.".to_string(), timestamp: Some("00:00:01.000".to_string()) },
//!     Cue { speaker: Some("Alice".to_string()), text: "How are you?".to_string(), timestamp: Some("00:00:02.000".to_string()) },
//!     Cue { speaker: Some("Bob".to_string()), text: "I'm fine!".to_string(), timestamp: Some("00:00:03.000".to_string()) },
//! ];
//!
//! let segments = consolidate_cues(&cues, "Unknown", TimestampMode::None);
//! // Result: 2 segments - Alice's text consolidated, Bob separate
//! assert_eq!(segments.len(), 2);
//! assert_eq!(segments[0].text, "Hello. How are you?");
//! ```

use crate::cli::TimestampMode;
use crate::parser::Cue;

/// Represents a consolidated speaker segment with speaker name, text, and optional timestamps.
#[derive(Debug, Clone, PartialEq)]
pub struct SpeakerSegment {
    /// The speaker's name (or custom label for unknown speakers)
    pub speaker: String,
    /// The consolidated text from all consecutive cues by this speaker
    pub text: String,
    /// Optional timestamp for the segment (based on TimestampMode)
    pub timestamp: Option<String>,
    /// Optional list of all timestamps from original cues (for TimestampMode::Each)
    pub timestamps: Vec<String>,
}

/// Consolidate a list of parsed cues into speaker segments.
///
/// This function groups consecutive cues from the same speaker into single segments,
/// joins their text intelligently, and handles timestamp inclusion based on the mode.
///
/// # Arguments
///
/// * `cues` - The list of parsed cues from a VTT document
/// * `unknown_speaker_label` - The label to use for cues without speaker attribution
/// * `timestamp_mode` - How to include timestamps in the output (None, First, or Each)
///
/// # Returns
///
/// A vector of `SpeakerSegment` structs with consolidated text per speaker turn.
///
/// # Example
///
/// ```rust,ignore
/// let cues = vec![
///     Cue { speaker: Some("Alice".to_string()), text: "Hello".to_string(), timestamp: Some("00:00:01.000".to_string()) },
///     Cue { speaker: Some("Alice".to_string()), text: "How are you?".to_string(), timestamp: Some("00:00:02.000".to_string()) },
///     Cue { speaker: Some("Bob".to_string()), text: "I'm fine.".to_string(), timestamp: Some("00:00:03.000".to_string()) },
/// ];
/// let segments = consolidate_cues(&cues, "Unknown", TimestampMode::First);
/// assert_eq!(segments.len(), 2); // Alice and Bob
/// ```
pub fn consolidate_cues(
    cues: &[Cue],
    unknown_speaker_label: &str,
    timestamp_mode: TimestampMode,
) -> Vec<SpeakerSegment> {
    let mut segments = Vec::new();
    let mut current_speaker: Option<String> = None;
    let mut current_texts = Vec::new();
    let mut current_timestamps = Vec::new();
    let mut first_timestamp: Option<String> = None;

    for cue in cues {
        // Skip empty or whitespace-only cues
        if cue.text.trim().is_empty() {
            continue;
        }

        // Determine the speaker for this cue (use label if None)
        let speaker = cue
            .speaker
            .clone()
            .unwrap_or_else(|| unknown_speaker_label.to_string());

        // Check if speaker changed
        let speaker_changed = current_speaker.as_ref() != Some(&speaker);

        if speaker_changed {
            // Save the previous segment if it exists
            if let Some(prev_speaker) = current_speaker.take() {
                let consolidated_text = join_texts(&current_texts);
                let segment_timestamp = match timestamp_mode {
                    TimestampMode::None => None,
                    TimestampMode::First => first_timestamp.clone(),
                    TimestampMode::Each => None, // Timestamps stored in timestamps vec
                };

                segments.push(SpeakerSegment {
                    speaker: prev_speaker,
                    text: consolidated_text,
                    timestamp: segment_timestamp,
                    timestamps: current_timestamps.clone(),
                });

                // Clear accumulators
                current_texts.clear();
                current_timestamps.clear();
            }

            // Start new segment
            current_speaker = Some(speaker);
            first_timestamp = cue.timestamp.clone();
        }

        // Add current cue to the segment
        current_texts.push(cue.text.clone());
        if let Some(ts) = &cue.timestamp {
            current_timestamps.push(ts.clone());
        }
    }

    // Save the final segment
    if let Some(speaker) = current_speaker {
        let consolidated_text = join_texts(&current_texts);
        let segment_timestamp = match timestamp_mode {
            TimestampMode::None => None,
            TimestampMode::First => first_timestamp,
            TimestampMode::Each => None, // Timestamps stored in timestamps vec
        };

        segments.push(SpeakerSegment {
            speaker,
            text: consolidated_text,
            timestamp: segment_timestamp,
            timestamps: current_timestamps,
        });
    }

    segments
}

/// Join multiple text segments intelligently with proper spacing.
///
/// This function joins text segments with single spaces, ensuring natural reading flow
/// while respecting sentence boundaries. It handles cases where segments may already
/// end with terminal punctuation. Empty or whitespace-only segments are skipped.
///
/// # Arguments
///
/// * `texts` - A slice of strings to join
///
/// # Returns
///
/// A single string with all non-empty segments joined by single spaces.
fn join_texts(texts: &[String]) -> String {
    let mut result = String::new();

    for text in texts.iter() {
        let text = text.trim();

        if text.is_empty() {
            continue;
        }

        if !result.is_empty() {
            // Add space between segments
            result.push(' ');
        }

        result.push_str(text);
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_consolidate_consecutive_same_speaker() {
        let cues = vec![
            Cue {
                speaker: Some("Alice".to_string()),
                text: "Hello there.".to_string(),
                timestamp: Some("00:00:01.000".to_string()),
            },
            Cue {
                speaker: Some("Alice".to_string()),
                text: "How are you?".to_string(),
                timestamp: Some("00:00:02.000".to_string()),
            },
            Cue {
                speaker: Some("Alice".to_string()),
                text: "I hope you're well.".to_string(),
                timestamp: Some("00:00:03.000".to_string()),
            },
        ];

        let segments = consolidate_cues(&cues, "Unknown", TimestampMode::None);

        assert_eq!(segments.len(), 1);
        assert_eq!(segments[0].speaker, "Alice");
        assert_eq!(
            segments[0].text,
            "Hello there. How are you? I hope you're well."
        );
        assert_eq!(segments[0].timestamp, None);
    }

    #[test]
    fn test_consolidate_different_speakers() {
        let cues = vec![
            Cue {
                speaker: Some("Alice".to_string()),
                text: "Hello.".to_string(),
                timestamp: Some("00:00:01.000".to_string()),
            },
            Cue {
                speaker: Some("Bob".to_string()),
                text: "Hi Alice!".to_string(),
                timestamp: Some("00:00:02.000".to_string()),
            },
            Cue {
                speaker: Some("Alice".to_string()),
                text: "How are you?".to_string(),
                timestamp: Some("00:00:03.000".to_string()),
            },
            Cue {
                speaker: Some("Bob".to_string()),
                text: "I'm good, thanks!".to_string(),
                timestamp: Some("00:00:04.000".to_string()),
            },
        ];

        let segments = consolidate_cues(&cues, "Unknown", TimestampMode::None);

        assert_eq!(segments.len(), 4);
        assert_eq!(segments[0].speaker, "Alice");
        assert_eq!(segments[0].text, "Hello.");
        assert_eq!(segments[1].speaker, "Bob");
        assert_eq!(segments[1].text, "Hi Alice!");
        assert_eq!(segments[2].speaker, "Alice");
        assert_eq!(segments[2].text, "How are you?");
        assert_eq!(segments[3].speaker, "Bob");
        assert_eq!(segments[3].text, "I'm good, thanks!");
    }

    #[test]
    fn test_consolidate_unknown_speaker_label() {
        let cues = vec![
            Cue {
                speaker: None,
                text: "This has no speaker.".to_string(),
                timestamp: Some("00:00:01.000".to_string()),
            },
            Cue {
                speaker: None,
                text: "Neither does this.".to_string(),
                timestamp: Some("00:00:02.000".to_string()),
            },
        ];

        let segments = consolidate_cues(&cues, "Narrator", TimestampMode::None);

        assert_eq!(segments.len(), 1);
        assert_eq!(segments[0].speaker, "Narrator");
        assert_eq!(segments[0].text, "This has no speaker. Neither does this.");
    }

    #[test]
    fn test_consolidate_skip_empty_cues() {
        let cues = vec![
            Cue {
                speaker: Some("Alice".to_string()),
                text: "Hello.".to_string(),
                timestamp: Some("00:00:01.000".to_string()),
            },
            Cue {
                speaker: Some("Alice".to_string()),
                text: "   ".to_string(), // Whitespace only
                timestamp: Some("00:00:02.000".to_string()),
            },
            Cue {
                speaker: Some("Alice".to_string()),
                text: "How are you?".to_string(),
                timestamp: Some("00:00:03.000".to_string()),
            },
        ];

        let segments = consolidate_cues(&cues, "Unknown", TimestampMode::None);

        assert_eq!(segments.len(), 1);
        assert_eq!(segments[0].speaker, "Alice");
        // The whitespace-only cue should be skipped
        assert_eq!(segments[0].text, "Hello. How are you?");
    }

    #[test]
    fn test_consolidate_text_joining() {
        let cues = vec![
            Cue {
                speaker: Some("Alice".to_string()),
                text: "First sentence.".to_string(),
                timestamp: None,
            },
            Cue {
                speaker: Some("Alice".to_string()),
                text: "Second sentence.".to_string(),
                timestamp: None,
            },
            Cue {
                speaker: Some("Alice".to_string()),
                text: "Third sentence.".to_string(),
                timestamp: None,
            },
        ];

        let segments = consolidate_cues(&cues, "Unknown", TimestampMode::None);

        assert_eq!(segments.len(), 1);
        // Sentences should be joined with single spaces
        assert_eq!(
            segments[0].text,
            "First sentence. Second sentence. Third sentence."
        );
    }

    #[test]
    fn test_consolidate_timestamp_mode_none() {
        let cues = vec![Cue {
            speaker: Some("Alice".to_string()),
            text: "Hello.".to_string(),
            timestamp: Some("00:00:01.000".to_string()),
        }];

        let segments = consolidate_cues(&cues, "Unknown", TimestampMode::None);

        assert_eq!(segments.len(), 1);
        assert_eq!(segments[0].timestamp, None);
    }

    #[test]
    fn test_consolidate_timestamp_mode_first() {
        let cues = vec![
            Cue {
                speaker: Some("Alice".to_string()),
                text: "Hello.".to_string(),
                timestamp: Some("00:00:01.000".to_string()),
            },
            Cue {
                speaker: Some("Alice".to_string()),
                text: "How are you?".to_string(),
                timestamp: Some("00:00:02.000".to_string()),
            },
            Cue {
                speaker: Some("Bob".to_string()),
                text: "I'm fine.".to_string(),
                timestamp: Some("00:00:03.000".to_string()),
            },
        ];

        let segments = consolidate_cues(&cues, "Unknown", TimestampMode::First);

        assert_eq!(segments.len(), 2);
        // First segment should have timestamp from first Alice cue
        assert_eq!(segments[0].timestamp, Some("00:00:01.000".to_string()));
        // Second segment should have timestamp from Bob's cue
        assert_eq!(segments[1].timestamp, Some("00:00:03.000".to_string()));
    }

    #[test]
    fn test_consolidate_timestamp_mode_each() {
        let cues = vec![
            Cue {
                speaker: Some("Alice".to_string()),
                text: "Hello.".to_string(),
                timestamp: Some("00:00:01.000".to_string()),
            },
            Cue {
                speaker: Some("Alice".to_string()),
                text: "How are you?".to_string(),
                timestamp: Some("00:00:02.000".to_string()),
            },
            Cue {
                speaker: Some("Alice".to_string()),
                text: "I hope you're well.".to_string(),
                timestamp: Some("00:00:03.000".to_string()),
            },
        ];

        let segments = consolidate_cues(&cues, "Unknown", TimestampMode::Each);

        assert_eq!(segments.len(), 1);
        // In Each mode, timestamp field is None, but timestamps vec contains all
        assert_eq!(segments[0].timestamp, None);
        assert_eq!(segments[0].timestamps.len(), 3);
        assert_eq!(segments[0].timestamps[0], "00:00:01.000");
        assert_eq!(segments[0].timestamps[1], "00:00:02.000");
        assert_eq!(segments[0].timestamps[2], "00:00:03.000");
    }

    #[test]
    fn test_join_texts() {
        assert_eq!(
            join_texts(&[
                "First.".to_string(),
                "Second.".to_string(),
                "Third.".to_string()
            ]),
            "First. Second. Third."
        );

        // Test with empty strings
        assert_eq!(
            join_texts(&["First.".to_string(), "".to_string(), "Third.".to_string()]),
            "First. Third."
        );

        // Test with whitespace
        assert_eq!(
            join_texts(&["  First.  ".to_string(), "  Second.  ".to_string()]),
            "First. Second."
        );

        // Test empty input
        assert_eq!(join_texts(&[]), "");
    }
}
