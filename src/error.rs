//! Error types and exit code handling for VTT to Markdown conversion.
//!
//! This module defines the custom error types used throughout the application
//! and maps them to BSD sysexits.h exit codes for consistent error reporting.
//!
//! # Example
//!
//! ```rust,ignore
//! use std::path::PathBuf;
//! use vtt_to_md::error::VttError;
//!
//! let error = VttError::FileNotFound {
//!     path: PathBuf::from("missing.vtt")
//! };
//! let exit_code = error.exit_code();
//! // Returns ExitCode with value 66 (EX_NOINPUT)
//! ```

#![allow(dead_code)]

use std::io;
use std::path::PathBuf;
use std::process::ExitCode;
use thiserror::Error;

/// Custom error type for VTT to Markdown conversion operations.
///
/// Each variant maps to a specific BSD sysexits.h exit code and provides
/// context about what went wrong.
#[derive(Error, Debug)]
pub enum VttError {
    /// Input file was not found.
    #[error("Input file not found: {path}")]
    FileNotFound { path: PathBuf },

    /// Permission denied when accessing a file.
    #[error("Permission denied: {path}")]
    PermissionDenied { path: PathBuf },

    /// Error parsing VTT file format.
    #[error("Failed to parse VTT file: {reason}")]
    ParseError { reason: String },

    /// Output file already exists and --force was not specified.
    #[error("Output file already exists: {path} (use --force to overwrite)")]
    OutputExists { path: PathBuf },

    /// Input and output paths are the same.
    #[error("Output path cannot be the same as input path: {path}")]
    SameFile { path: PathBuf },

    /// Error writing output file.
    #[error("Failed to write output file: {path}")]
    WriteError { path: PathBuf, source: io::Error },

    /// General I/O error.
    #[error("I/O error: {0}")]
    IoError(#[from] io::Error),

    /// Invalid command-line usage.
    #[error("Invalid usage: {reason}")]
    UsageError { reason: String },
}

impl VttError {
    /// Map error to appropriate BSD sysexits.h exit code.
    ///
    /// # Exit Code Mapping
    ///
    /// - `64` (EX_USAGE): Invalid command-line usage or conflicting arguments
    /// - `65` (EX_DATAERR): Invalid VTT file format or parse errors
    /// - `66` (EX_NOINPUT): Input file not found
    /// - `73` (EX_CANTCREAT): Output file already exists without --force
    /// - `74` (EX_IOERR): General I/O or write errors
    /// - `77` (EX_NOPERM): Permission denied when accessing files
    pub fn exit_code(&self) -> ExitCode {
        let code = match self {
            VttError::UsageError { .. } => 64,       // EX_USAGE
            VttError::ParseError { .. } => 65,       // EX_DATAERR
            VttError::FileNotFound { .. } => 66,     // EX_NOINPUT
            VttError::OutputExists { .. } => 73,     // EX_CANTCREAT
            VttError::WriteError { .. } => 74,       // EX_IOERR
            VttError::IoError(_) => 74,              // EX_IOERR
            VttError::PermissionDenied { .. } => 77, // EX_NOPERM
            VttError::SameFile { .. } => 64,         // EX_USAGE
        };
        ExitCode::from(code)
    }
}
