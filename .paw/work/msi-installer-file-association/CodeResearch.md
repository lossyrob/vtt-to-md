---
date: 2025-11-18T11:45:11-05:00
git_commit: 2e995ae33c0a30b294d37a635cdde2ef2f8aabba
branch: feature/installer
repository: vtt-to-md
topic: "MSI Installer with File Association for Windows"
tags: [research, codebase, msi-installer, wix, file-association, windows, rust, cargo]
status: complete
last_updated: 2025-11-18
---

# Research: MSI Installer with File Association for Windows

**Date**: 2025-11-18 11:45:11 -05:00  
**Git Commit**: 2e995ae33c0a30b294d37a635cdde2ef2f8aabba  
**Branch**: feature/installer  
**Repository**: vtt-to-md

## Research Question

How to create an MSI installer for the vtt-to-md Rust CLI application that enables:
1. Windows installation with standard MSI package
2. Optional .vtt file association allowing double-click to convert files
3. Automated GitHub Actions release workflow
4. Version synchronization between Cargo.toml and git tags

## Summary

The vtt-to-md application is a Rust CLI tool that converts WebVTT transcript files to Markdown. The application currently has no installer infrastructure and no CI/CD automation. Research shows:

**Current Application Architecture:**
- Simple single-file input CLI accepting a file path argument (`src/cli.rs:21`)
- Automatically derives output path by replacing .vtt extension with .md (`src/cli.rs:111`)
- Creates output file in the same directory as the input file by default
- Application handles file paths correctly for installer use case - accepts absolute paths via command line

**Build Infrastructure Gaps:**
- No existing WiX configuration (no `wix/` directory or `.wxs` files)
- No GitHub Actions workflows (no `.github/` directory)
- Version is hardcoded in `Cargo.toml:3` as `0.1.0`
- No automation for version bumping or release management

**Recommended Tooling:**
- **cargo-wix**: Cargo subcommand for creating MSI installers from Rust projects
- **WiX Toolset**: Required underlying technology for MSI generation
- File association pattern: ProgId + Extension + Verb configuration in WiX XML

## Detailed Findings

### Application Entry Point and File Handling

**Main Entry Point** (`src/main.rs:12-30`)
```rust
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

    // Run the conversion
    if let Err(e) = run_conversion(&args) {
        eprintln!("Error: {}", e);
        return e.exit_code();
    }

    ExitCode::SUCCESS
}
```

The application accepts a single file path as input and optionally an output path.

**File Path Handling** (`src/cli.rs:21-30`)
```rust
pub struct Args {
    /// Path to the input VTT file
    #[arg(value_name = "INPUT", help = "Path to the input VTT file")]
    pub input: PathBuf,

    /// Path to the output Markdown file (defaults to INPUT with .md extension)
    #[arg(value_name = "OUTPUT", help = "Path to the output Markdown file")]
    pub output: Option<PathBuf>,
```

**Output Path Derivation** (`src/cli.rs:89-93`)
```rust
// Derive output path if not specified and not using stdout
if self.output.is_none() && !self.stdout {
    self.output = Some(derive_output_path(&self.input));
}
```

**Default Output Location** (`src/cli.rs:111-113`)
```rust
fn derive_output_path(input: &Path) -> PathBuf {
    input.with_extension("md")
}
```

The output file is created in the same directory as the input file by default, which is the ideal behavior for file association (double-clicking a .vtt file creates a .md file alongside it).

### Application Behavior with File Arguments

**File Reading** (`src/parser.rs:48-63`)
```rust
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
```

The parser accepts any path (relative or absolute) and handles common file errors appropriately.

**File Writing** (`src/markdown.rs:57-92`)
```rust
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
```

File writing has safety guards (won't overwrite by default) which may need consideration for file association use case. When invoked via double-click, the application should probably use `--force` flag to overwrite existing .md files.

### Current Build Configuration

**Cargo Configuration** (`Cargo.toml:1-11`)
```toml
[package]
name = "vtt-to-md"
version = "0.1.0"
edition = "2024"

[dependencies]
clap = { version = "4.5", features = ["derive", "color", "suggestions"] }
thiserror = "1.0"
anyhow = "1.0"
regex = "1.10"
unicode-normalization = "0.1"
```

The version is hardcoded as `0.1.0`. For automated releases, this needs to be synchronized with git tags.

**Rust Toolchain** (`rust-toolchain.toml:1-2`)
```toml
[toolchain]
channel = "stable"
```

Uses stable Rust channel - no special requirements.

**Binary Target** (`Cargo.toml:1-2`)
```toml
[package]
name = "vtt-to-md"
```

Binary name is `vtt-to-md` (or `vtt-to-md.exe` on Windows).

### No Existing Installer Infrastructure

**Missing Components:**
- No `wix/` directory or `.wxs` WiX source files
- No `.github/workflows/` directory or CI/CD automation
- No `cargo-wix` configuration
- No installer-related scripts or build files

**Gitignore** (`.gitignore:1`)
```
/target
```

Only ignores build artifacts. Will need to add MSI output directories (e.g., `target/wix/`).

### Integration Test Patterns

The application has comprehensive integration tests that execute the compiled binary (`tests/integration_test.rs:20-33`):

```rust
fn get_vtt_to_md_path() -> PathBuf {
    let mut path = std::env::current_exe().expect("Failed to get current executable path");
    path.pop(); // Remove test executable name
    path.pop(); // Remove 'deps' directory

    #[cfg(target_os = "windows")]
    path.push("vtt-to-md.exe");

    #[cfg(not(target_os = "windows"))]
    path.push("vtt-to-md");

    path
}
```

Tests demonstrate the application works correctly when invoked with file paths, including paths with spaces (`tests/integration_test.rs:272-289`).

## Code References

### CLI and File Handling
- `src/main.rs:12-30` - Main entry point accepting file path arguments
- `src/cli.rs:21-30` - Args struct with input/output PathBuf fields
- `src/cli.rs:89-93` - Output path validation and derivation
- `src/cli.rs:111-113` - Default output path derivation (same directory, .md extension)
- `src/cli.rs:116-129` - Path equality checking for input/output validation

### File I/O Operations
- `src/parser.rs:48-77` - VTT file parsing with error handling
- `src/markdown.rs:57-92` - Markdown file writing with overwrite protection
- `src/error.rs:19-44` - Error types for file operations (FileNotFound, PermissionDenied, OutputExists, etc.)
- `src/error.rs:46-62` - Exit code mapping using BSD sysexits.h codes

### Testing Patterns
- `tests/integration_test.rs:20-33` - Binary path resolution for Windows/Linux
- `tests/integration_test.rs:272-289` - Path with spaces test
- `tests/integration_test.rs:294-311` - Custom output path test

### Project Metadata
- `Cargo.toml:1-11` - Package configuration and version
- `rust-toolchain.toml:1-2` - Toolchain requirements (stable)
- `CHANGELOG.md:9-10` - Version history and release notes format

## Architecture Documentation

### Application Flow for File Association

When a .vtt file is double-clicked with file association configured:

1. Windows shell invokes: `vtt-to-md.exe "C:\Path\To\File.vtt"`
2. Main entry point parses arguments (`src/main.rs:14`)
3. Args validation derives output path as `C:\Path\To\File.md` (`src/cli.rs:91`)
4. Parser reads and parses VTT file (`src/parser.rs:48`)
5. Consolidator merges speaker segments (`src/consolidator.rs:55`)
6. Markdown formatter creates output text (`src/markdown.rs:33`)
7. Writer creates `File.md` in same directory (`src/markdown.rs:57`)

**Key Requirement for File Association:**
The WiX configuration must pass the file path as a quoted argument: `"%1"` where `%1` is the absolute path to the clicked file.

### Current Application Constraints

**Single File Processing:**
- Application accepts exactly one input file
- No batch processing or directory traversal
- Perfect fit for file association use case

**Output Location:**
- Always same directory as input (when output not specified)
- Requires write permission in source directory
- May fail if .vtt file is on read-only media (CD, network share)

**Safety Features:**
- Won't overwrite existing .md file without `--force` flag
- For file association, installer should configure verb as: `vtt-to-md.exe --force "%1"`

## Research Resources

### Rust MSI Installer Ecosystem

**cargo-wix** (Primary Tool)
- Cargo subcommand for building MSI installers
- Uses WiX Toolset under the hood
- Command: `cargo wix init` creates initial `wix/` folder structure
- MSI output location: `target/wix/`
- Documentation: https://volks73.github.io/cargo-wix/

**WiX Toolset** (Required Dependency)
- Industry-standard Windows installer framework
- Requires local installation on build machine
- GitHub Actions runners don't include WiX by default (requires setup step)

### File Association Pattern in WiX

**ProgId Configuration:**
```xml
<ProgId Id="VttToMd.VttFile" Description="VTT Transcript File">
    <Extension Id="vtt" ContentType="text/vtt">
        <Verb Id="open" Command="Convert to Markdown" 
              TargetFile="VttToMdExe" 
              Argument="--force &quot;%1&quot;" />
    </Extension>
</ProgId>
```

**Key Elements:**
- `ProgId`: Associates file type with application
- `Extension Id="vtt"`: The file extension to register
- `Verb`: Action when file is opened (double-click)
- `Argument="--force &quot;%1&quot;"`: Pass file path with force flag
- `%1`: Windows placeholder for clicked file's absolute path

Reference: https://www.advancedinstaller.com/versus/wix-toolset/register-file-association-with-wix.html

### GitHub Actions Automation Pattern

**Workflow Trigger:**
```yaml
on:
  push:
    tags:
      - 'v*.*.*'
```

**WiX Setup in GitHub Actions:**
- WiX Toolset must be installed as a step
- Requires Windows runner: `runs-on: windows-latest`
- cargo-wix must be installed: `cargo install cargo-wix`

**Version Synchronization:**
- Read version from git tag (e.g., `v1.0.0`)
- Update `Cargo.toml` version field or use environment variable
- Pass version to cargo-wix for MSI metadata

Reference: GitHub discussions on cargo-wix automation

## Open Questions

1. **File Association Scope:** Should file association be per-user or per-machine?
   - Per-machine requires admin elevation during install
   - Per-user is safer but may not work for all Windows users

2. **Optional File Association:** Issue states file association should be optional
   - WiX supports checkbox UI during installation
   - Requires conditional component installation based on checkbox

3. **Version Synchronization Strategy:**
   - Option A: Keep Cargo.toml at 0.0.0, override at build time from tag
   - Option B: Manually update Cargo.toml before tagging
   - Option C: Automated PR to bump version after successful release

4. **Icon for File Association:**
   - Application currently has no icon
   - File association typically shows custom icon for registered file type
   - May want to add application icon and .vtt file icon

5. **Build Machine Requirements:**
   - WiX Toolset requires Windows build environment
   - GitHub Actions Windows runners support this
   - Local builds require developer to install WiX

6. **Force Flag Behavior:**
   - Should file association always use `--force` flag?
   - Risk: Users might not expect existing .md files to be overwritten
   - Alternative: Use `--no-clobber` but provide user feedback when skipped

## Next Steps for Implementation Planning

The implementation plan should address:

1. **cargo-wix Integration:**
   - Initialize WiX configuration with `cargo wix init`
   - Customize `wix/main.wxs` for file association
   - Configure ProgId and Extension elements
   - Add optional checkbox for file association

2. **Version Management:**
   - Decide on version synchronization strategy
   - Implement mechanism to extract version from git tag
   - Update Cargo.toml or pass version to cargo-wix

3. **GitHub Actions Workflow:**
   - Create `.github/workflows/release.yml`
   - Trigger on tag push (pattern: `v*.*.*`)
   - Install WiX Toolset
   - Build MSI with cargo-wix
   - Upload MSI as release artifact

4. **Testing Strategy:**
   - Test MSI installation on clean Windows system
   - Verify file association registration
   - Test double-click behavior on .vtt files
   - Verify uninstallation cleans up file association

5. **Documentation Updates:**
   - Update README.md with installer download instructions
   - Document release process for maintainers
   - Add instructions for local MSI builds

