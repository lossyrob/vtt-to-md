# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.0] - 2025-11-18

### Added
- Initial release of VTT to MD CLI tool for converting WebVTT transcript files to Markdown format
- Speaker consolidation that merges consecutive cues from the same speaker into coherent paragraphs
- Multi-line voice tag support for properly parsing VTT files with text spanning multiple lines
- Timestamp sorting to handle out-of-order cues common in Teams transcripts
- Smart unknown speaker filtering that automatically removes cues without attribution in Teams-style VTT files
- Flexible timestamp modes (none, first, each) for including timestamps in output
- Command-line flags: --force, --no-clobber, --stdout, --filter-unknown, --no-filter-unknown
- Options: --unknown-speaker (custom label), --include-timestamps (mode selection)
- Safe-by-default behavior that prevents overwriting existing files without explicit --force flag
- Cross-platform support (Windows, Linux, macOS) with no runtime dependencies
- Comprehensive error handling with BSD sysexits.h exit codes
- Support for VTT files from Microsoft Teams, Zoom, and Google Meet
