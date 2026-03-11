//! Filler word and speech disfluency removal.
//!
//! This module removes common filler words and speech disfluencies from transcript
//! text. It handles fillers at the start, middle, and end of sentences, and cleans
//! up punctuation artifacts left behind after removal.

use regex::Regex;

/// Filler patterns (case-insensitive, word-boundary-delimited).
///
/// Extended set includes basic disfluencies plus common discourse markers.
/// Ambiguous words like "like", "so", "right", "OK" are intentionally excluded.
const FILLER_PATTERNS: &[&str] = &[
    // Multi-word phrases (must come before single words)
    r"uh huh",
    r"you know",
    r"i mean",
    // Single words
    r"uh",
    r"um",
    r"mhm",
    r"hmm+",
    r"mm+",
    r"ah",
    r"er",
    r"huh",
];

/// Remove filler words from the given text.
///
/// Handles fillers appearing:
/// - As the entire text: `"Uh."` → `""`
/// - At the start: `"Uh, I think"` → `"I think"`
/// - In the middle: `"on, uh, performance"` → `"on, performance"`
/// - At the end: `"that's fine, uh"` → `"that's fine"`
///
/// Cleans up artifacts: double commas, orphaned punctuation, extra whitespace.
pub fn remove_fillers(text: &str) -> String {
    if text.trim().is_empty() {
        return String::new();
    }

    // Build a combined regex: match filler words optionally surrounded by
    // comma+space or just space, with optional trailing period/comma.
    // Pattern matches: optional leading ", " + filler + optional trailing ","/"."/space
    let filler_alternation = FILLER_PATTERNS.join("|");
    let pattern = format!(
        r"(?i)(?:,\s*)?(?:\b(?:{filler})\b)(?:\s*[,.])?",
        filler = filler_alternation
    );
    let filler_regex = Regex::new(&pattern).unwrap();

    let result = filler_regex.replace_all(text, "");

    // Clean up artifacts
    let result = cleanup_artifacts(&result);

    result
}

/// Clean up text artifacts left after filler removal.
fn cleanup_artifacts(text: &str) -> String {
    // Collapse multiple spaces
    let multi_space = Regex::new(r" {2,}").unwrap();
    let text = multi_space.replace_all(text, " ");

    // Remove orphaned commas: ", ," → ","
    let double_comma = Regex::new(r",\s*,").unwrap();
    let text = double_comma.replace_all(&text, ",");

    // Remove leading comma+space at start of text
    let leading_comma = Regex::new(r"^\s*,\s*").unwrap();
    let text = leading_comma.replace(&text, "");

    // Remove trailing comma/space at end of text
    let trailing_comma = Regex::new(r"\s*,\s*$").unwrap();
    let text = trailing_comma.replace(&text, "");

    let text = text.trim().to_string();

    // Collapse any remaining multiple spaces from chained removals
    let multi_space = Regex::new(r" {2,}").unwrap();
    let text = multi_space.replace_all(&text, " ");

    let text = text.trim().to_string();

    // Capitalize after sentence boundaries (". ", "! ", "? ")
    let text = capitalize_after_sentence_boundaries(&text);

    // Capitalize the very first letter of the text
    capitalize_first(&text)
}

/// Capitalize the first letter after each sentence-ending punctuation (. ! ?).
fn capitalize_after_sentence_boundaries(s: &str) -> String {
    let re = Regex::new(r"([.!?])\s+([a-z])").unwrap();
    re.replace_all(s, |caps: &regex::Captures| {
        format!(
            "{} {}",
            &caps[1],
            caps[2].to_uppercase()
        )
    })
    .to_string()
}

/// Capitalize the first character, but only if it's a simple lowercase ASCII letter.
/// Preserves intentional lowercase starts (e.g., "iPhone", "eBay").
fn capitalize_first(s: &str) -> String {
    if s.is_empty() {
        return String::new();
    }

    let mut chars = s.chars();
    match chars.next() {
        None => String::new(),
        Some(c) if c.is_ascii_lowercase() => {
            // Only capitalize if the second char is also lowercase or non-alphabetic.
            // This heuristic preserves camelCase brand names like "iPhone".
            let rest = chars.as_str();
            let second_is_upper = rest.chars().next().is_some_and(|c2| c2.is_uppercase());
            if second_is_upper {
                // Likely a brand name like "iPhone" — don't capitalize
                format!("{c}{rest}")
            } else {
                let upper: String = c.to_uppercase().collect();
                upper + rest
            }
        }
        Some(c) => {
            format!("{c}{}", chars.as_str())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- Entire text is filler ---

    #[test]
    fn test_entire_text_single_filler() {
        assert_eq!(remove_fillers("Uh."), "");
        assert_eq!(remove_fillers("Um"), "");
        assert_eq!(remove_fillers("Mhm."), "");
        assert_eq!(remove_fillers("Hmm"), "");
        assert_eq!(remove_fillers("Uh huh."), "");
    }

    #[test]
    fn test_entire_text_filler_case_insensitive() {
        assert_eq!(remove_fillers("UH"), "");
        assert_eq!(remove_fillers("um."), "");
        assert_eq!(remove_fillers("MHM"), "");
    }

    // --- Filler at start of text ---

    #[test]
    fn test_filler_at_start_with_comma() {
        assert_eq!(remove_fillers("Uh, I think so"), "I think so");
        assert_eq!(remove_fillers("Um, that's right"), "That's right");
    }

    #[test]
    fn test_filler_at_start_without_comma() {
        assert_eq!(remove_fillers("Uh I think so"), "I think so");
    }

    #[test]
    fn test_multi_word_filler_at_start() {
        assert_eq!(
            remove_fillers("You know, it works great"),
            "It works great"
        );
        assert_eq!(
            remove_fillers("I mean, we should try"),
            "We should try"
        );
    }

    // --- Filler in middle of text ---

    #[test]
    fn test_filler_mid_sentence() {
        assert_eq!(
            remove_fillers("I think, uh, we should go"),
            "I think we should go"
        );
        assert_eq!(
            remove_fillers("The thing, um, is important"),
            "The thing is important"
        );
    }

    #[test]
    fn test_multi_word_filler_mid_sentence() {
        assert_eq!(
            remove_fillers("It's, you know, pretty good"),
            "It's pretty good"
        );
        assert_eq!(
            remove_fillers("We could, I mean, try something"),
            "We could try something"
        );
    }

    // --- Filler at end of text ---

    #[test]
    fn test_filler_at_end() {
        assert_eq!(remove_fillers("That's fine, uh"), "That's fine");
        assert_eq!(remove_fillers("That's fine, um."), "That's fine");
    }

    // --- Multiple fillers ---

    #[test]
    fn test_multiple_fillers() {
        assert_eq!(
            remove_fillers("Uh, I think, um, we should, uh, go"),
            "I think we should go"
        );
    }

    // --- No fillers (should be unchanged) ---

    #[test]
    fn test_no_fillers_unchanged() {
        assert_eq!(
            remove_fillers("Hello, how are you?"),
            "Hello, how are you?"
        );
        assert_eq!(
            remove_fillers("This is a normal sentence."),
            "This is a normal sentence."
        );
    }

    // --- Fillers should not match inside words ---

    #[test]
    fn test_filler_word_boundaries() {
        assert_eq!(remove_fillers("The umbrella is here"), "The umbrella is here");
        assert_eq!(remove_fillers("Human beings are kind"), "Human beings are kind");
    }

    // --- Empty/whitespace input ---

    #[test]
    fn test_empty_input() {
        assert_eq!(remove_fillers(""), "");
        assert_eq!(remove_fillers("   "), "");
    }

    // --- Extended set fillers ---

    #[test]
    fn test_you_know_i_mean() {
        assert_eq!(
            remove_fillers("It's, you know, pretty good"),
            "It's pretty good"
        );
        assert_eq!(
            remove_fillers("I mean, we should try"),
            "We should try"
        );
    }

    // --- Hmm/mm variants ---

    #[test]
    fn test_hmm_variants() {
        assert_eq!(remove_fillers("Hmmm, that's interesting"), "That's interesting");
        assert_eq!(remove_fillers("Mmm, yes"), "Yes");
    }

    // --- Capitalize after removal ---

    #[test]
    fn test_capitalizes_first_letter_after_removal() {
        assert_eq!(remove_fillers("Uh, it's fine"), "It's fine");
        assert_eq!(remove_fillers("Um, the thing is"), "The thing is");
    }

    // --- Real transcript examples ---

    #[test]
    fn test_real_transcript_example() {
        assert_eq!(
            remove_fillers("Mhm. Uh, it it's fine. Uh, working on, uh, you know, performance dashboard."),
            "It it's fine. Working on performance dashboard."
        );
    }

    // --- Sentence boundary capitalization ---

    #[test]
    fn test_capitalizes_after_sentence_boundary() {
        assert_eq!(
            remove_fillers("Done. um, next topic."),
            "Done. Next topic."
        );
    }

    // --- Brand name preservation ---

    #[test]
    fn test_preserves_brand_name_capitalization() {
        assert_eq!(
            remove_fillers("Um, iPhone is great"),
            "iPhone is great"
        );
    }

    // --- "kind of" / "sort of" are NOT removed (legitimate uses) ---

    #[test]
    fn test_kind_of_sort_of_preserved() {
        assert_eq!(
            remove_fillers("What kind of car is it?"),
            "What kind of car is it?"
        );
        assert_eq!(
            remove_fillers("She sort of agreed"),
            "She sort of agreed"
        );
    }
}
