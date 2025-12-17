//! VTT to Markdown converter - command-line tool for converting WebVTT transcripts to readable Markdown.

mod cli;
mod config;
mod consolidator;
mod error;
mod markdown;
mod parser;

use clap::Parser;
use cli::Args;
use std::process::ExitCode;

fn main() -> ExitCode {
    // Parse command-line arguments
    let mut args = match Args::try_parse() {
        Ok(args) => args,
        Err(e) => {
            // Clap handles printing error messages and help text
            e.exit();
        }
    };

    // Validate arguments
    if let Err(e) = args.validate() {
        eprintln!("Error: {}", e);
        return e.exit_code();
    }

    // Resolve effective include_timestamps (CLI > env > config > built-in default)
    let mut warnings = Vec::new();

    if let Some(setting) = args.include_timestamps {
        if setting.is_legacy {
            warnings.push(
                "--include-timestamps uses legacy value (none|first|each); please migrate to true/false or omit the value"
                    .to_string(),
            );
        }
    }

    let (configured_default, config_warnings) = if args.include_timestamps.is_some() {
        (None, Vec::new())
    } else {
        config::resolve_default_include_timestamps()
    };

    warnings.extend(config_warnings);

    for warning in warnings {
        eprintln!("Warning: {warning}");
    }

    let effective_include_timestamps = args
        .include_timestamps
        .map(|setting| setting.value)
        .unwrap_or(configured_default.unwrap_or(false));

    // Run the conversion
    if let Err(e) = run_conversion(&args, effective_include_timestamps) {
        eprintln!("Error: {}", e);
        return e.exit_code();
    }

    ExitCode::SUCCESS
}

/// Run the VTT to Markdown conversion pipeline.
fn run_conversion(args: &Args, include_timestamps: bool) -> Result<(), error::VttError> {
    // Parse the VTT file
    let vtt_document = parser::VttDocument::parse(&args.input)?;

    // Determine if we should filter unknown speakers:
    // - Explicitly enabled with --filter-unknown
    // - OR auto-enabled for Teams format (has voice tags) unless disabled with --no-filter-unknown
    let should_filter = args.filter_unknown 
        || (vtt_document.has_voice_tags && !args.no_filter_unknown);

    // Filter cues if requested or auto-detected
    let cues = if should_filter {
        vtt_document
            .cues
            .into_iter()
            .filter(|cue| cue.speaker.is_some())
            .collect()
    } else {
        vtt_document.cues
    };

    // Consolidate speaker segments
    let segments = consolidator::consolidate_cues(&cues, &args.unknown_speaker, include_timestamps);

    // Format as Markdown
    let markdown_content = markdown::format_markdown(&segments, include_timestamps);

    // Write output (either to file or stdout)
    if args.stdout {
        markdown::write_markdown_stdout(&markdown_content)?;
    } else if let Some(output_path) = args.get_output_path() {
        markdown::write_markdown_file(&markdown_content, output_path, args.force, args.no_clobber)?;
    }

    Ok(())
}
