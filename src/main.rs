//! VTT to Markdown converter - command-line tool for converting WebVTT transcripts to readable Markdown.

mod cli;
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

    // Placeholder for conversion logic (will be implemented in later phases)
    println!("VTT to Markdown converter");
    println!("Input file: {}", args.input.display());
    if let Some(output) = args.get_output_path() {
        println!("Output file: {}", output.display());
    } else {
        println!("Output: stdout");
    }
    println!("Force overwrite: {}", args.force);
    println!("No clobber: {}", args.no_clobber);
    println!("Unknown speaker label: {}", args.unknown_speaker);
    println!("Timestamp mode: {:?}", args.include_timestamps);

    ExitCode::SUCCESS
}
