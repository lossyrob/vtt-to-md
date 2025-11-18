# MSI Installer with File Association Implementation Plan

## Overview

Create an MSI installer for the vtt-to-md Rust CLI application that enables Windows installation with an optional .vtt file association. The installer will use cargo-wix and WiX Toolset, with automated GitHub Actions release workflow triggered by git tags. A key enhancement is adding browser-style auto-increment filename collision handling so file association doesn't require the --force flag.

## Current State Analysis

**Application Architecture:**
- Rust CLI tool that converts WebVTT transcript files to Markdown (`src/main.rs:12-30`)
- Single file input via command line: `vtt-to-md.exe input.vtt`
- Automatically derives output path by replacing .vtt with .md extension (`src/cli.rs:139`)
- Creates output file in the same directory as input file
- Currently fails if output file exists (unless `--force` or `--no-clobber` flags used) (`src/markdown.rs:113-125`)

**Build Infrastructure:**
- No WiX configuration (no `wix/` directory)
- No GitHub Actions workflows (no `.github/workflows/` directory)
- Version hardcoded at `0.1.0` in `Cargo.toml:3`
- `.gitignore` only ignores `/target` directory

**Key Constraint:**
File association will invoke the application with an absolute path to the clicked file. Without auto-increment, users would need to manually delete existing .md files or use --force (which overwrites without warning). Browser-style auto-increment provides better UX.

## Desired End State

After this plan is complete:

1. **Auto-increment filename collision handling** implemented as default behavior
2. **WiX configuration** exists in `wix/` directory with file association support
3. **GitHub Actions workflow** automatically builds MSI on git tag push
4. **MSI installer** can be downloaded from GitHub releases
5. **Optional .vtt file association** (per-user, checkbox during install, default checked)
6. Double-clicking a .vtt file creates `filename.md`, `filename (1).md`, etc. without overwriting

### Verification:
- `cargo test` passes with new auto-increment tests
- `cargo wix` successfully builds MSI installer
- Manual install test: MSI installs, file association works, creates incremented files
- GitHub Actions workflow triggers on tag push and uploads MSI artifact

## What We're NOT Doing

- Not implementing per-machine file association (staying with per-user)
- Not making file association mandatory (keeping it optional with checkbox)
- Not creating custom icons for the application or file type
- Not implementing uninstaller notification/confirmation dialogs
- Not supporting other platforms (Linux/macOS package managers)
- Not implementing batch file processing features
- Not adding GUI for the application

## Implementation Approach

The implementation is divided into 4 phases that build incrementally:

1. **Phase 1**: Implement auto-increment filename feature as new default behavior
2. **Phase 2**: Initialize WiX configuration and create base MSI installer
3. **Phase 3**: Add optional per-user .vtt file association to installer
4. **Phase 4**: Create automated GitHub Actions release workflow

Each phase is independently testable and can be verified before proceeding to the next.

---

## Phase 1: Add Auto-Increment Filename Feature

### Overview
Implement browser-style filename collision handling where repeated conversions of the same file create `meeting.md`, `meeting (1).md`, `meeting (2).md`, etc. This becomes the default behavior, replacing the current error when output file exists.

### Changes Required:

#### 1. Add auto-increment logic to cli.rs
**File**: `src/cli.rs`
**Changes**: Add new function to find next available filename and update `derive_output_path()`

After the `derive_output_path()` function (around line 139), add:

```rust
/// Derive output path from input path by replacing extension with .md
fn derive_output_path(input: &Path) -> PathBuf {
    let base_output = input.with_extension("md");
    find_available_path(&base_output)
}

/// Find next available path by adding (N) suffix if file exists.
///
/// Given path/to/file.md, tries:
/// - path/to/file.md
/// - path/to/file (1).md
/// - path/to/file (2).md
/// etc.
fn find_available_path(base_path: &Path) -> PathBuf {
    if !base_path.exists() {
        return base_path.to_path_buf();
    }

    let parent = base_path.parent().unwrap_or_else(|| Path::new(""));
    let extension = base_path.extension().and_then(|s| s.to_str()).unwrap_or("");
    let stem = base_path.file_stem().and_then(|s| s.to_str()).unwrap_or("");

    for i in 1..=9999 {
        let new_name = if extension.is_empty() {
            format!("{} ({})", stem, i)
        } else {
            format!("{} ({}).{}", stem, i, extension)
        };
        let new_path = parent.join(new_name);
        if !new_path.exists() {
            return new_path;
        }
    }

    // Fallback: if we somehow exhaust 9999 attempts, return the base path
    // (will likely fail during write, but this is an extreme edge case)
    base_path.to_path_buf()
}
```

#### 2. Add --no-auto-increment flag for backwards compatibility
**File**: `src/cli.rs`
**Changes**: Add new flag to Args struct

In the Args struct (around line 69), add the new flag:

```rust
    /// Disable automatic filtering of unknown speakers for Teams-style VTT files
    #[arg(
        long,
        conflicts_with = "filter_unknown",
        help = "Disable automatic filtering of unknown speakers for Teams-style VTT files"
    )]
    pub no_filter_unknown: bool,

    /// Disable auto-increment of output filename on collision
    #[arg(
        long,
        help = "Disable auto-increment of output filename when file exists (use with --force to overwrite)"
    )]
    pub no_auto_increment: bool,

    /// Timestamp inclusion mode
    #[arg(
```

#### 3. Update validate() to conditionally use auto-increment
**File**: `src/cli.rs`
**Changes**: Modify validate() method to respect no_auto_increment flag

In the `validate()` method (around line 109), update the output path derivation:

```rust
    pub fn validate(&mut self) -> Result<(), VttError> {
        // Derive output path if not specified and not using stdout
        if self.output.is_none() && !self.stdout {
            if self.no_auto_increment {
                // Old behavior: simple extension replacement
                self.output = Some(self.input.with_extension("md"));
            } else {
                // New default: auto-increment on collision
                self.output = Some(derive_output_path(&self.input));
            }
        }

        // Check if input and output are the same file
```

#### 4. Add comprehensive tests for auto-increment logic
**File**: `tests/integration_test.rs`
**Changes**: Add new test cases for auto-increment behavior

Add tests at the end of the integration tests file:

```rust
#[test]
fn test_auto_increment_filename() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let input_vtt = temp_dir.path().join("meeting.vtt");
    
    // Create a simple VTT file
    fs::write(&input_vtt, SIMPLE_VTT).expect("Failed to write VTT file");
    
    // First conversion: creates meeting.md
    let output = Command::new(get_vtt_to_md_path())
        .arg(&input_vtt)
        .output()
        .expect("Failed to execute vtt-to-md");
    
    assert!(output.status.success(), "First conversion failed");
    
    let first_output = temp_dir.path().join("meeting.md");
    assert!(first_output.exists(), "meeting.md should exist");
    
    // Second conversion: should create meeting (1).md
    let output = Command::new(get_vtt_to_md_path())
        .arg(&input_vtt)
        .output()
        .expect("Failed to execute vtt-to-md");
    
    assert!(output.status.success(), "Second conversion failed");
    
    let second_output = temp_dir.path().join("meeting (1).md");
    assert!(second_output.exists(), "meeting (1).md should exist");
    
    // Third conversion: should create meeting (2).md
    let output = Command::new(get_vtt_to_md_path())
        .arg(&input_vtt)
        .output()
        .expect("Failed to execute vtt-to-md");
    
    assert!(output.status.success(), "Third conversion failed");
    
    let third_output = temp_dir.path().join("meeting (2).md");
    assert!(third_output.exists(), "meeting (2).md should exist");
    
    // Verify all three files exist and are different
    assert!(first_output.exists() && second_output.exists() && third_output.exists());
}

#[test]
fn test_no_auto_increment_flag() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let input_vtt = temp_dir.path().join("meeting.vtt");
    let output_md = temp_dir.path().join("meeting.md");
    
    // Create a simple VTT file
    fs::write(&input_vtt, SIMPLE_VTT).expect("Failed to write VTT file");
    
    // First conversion: creates meeting.md
    let output = Command::new(get_vtt_to_md_path())
        .arg(&input_vtt)
        .output()
        .expect("Failed to execute vtt-to-md");
    
    assert!(output.status.success());
    assert!(output_md.exists());
    
    // Second conversion with --no-auto-increment should fail
    let output = Command::new(get_vtt_to_md_path())
        .arg("--no-auto-increment")
        .arg(&input_vtt)
        .output()
        .expect("Failed to execute vtt-to-md");
    
    assert!(!output.status.success(), "Should fail when output exists");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("already exists") || stderr.contains("OutputExists"));
    
    // Should succeed with --force
    let output = Command::new(get_vtt_to_md_path())
        .arg("--no-auto-increment")
        .arg("--force")
        .arg(&input_vtt)
        .output()
        .expect("Failed to execute vtt-to-md");
    
    assert!(output.status.success(), "Should succeed with --force flag");
}

#[test]
fn test_explicit_output_skips_auto_increment() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let input_vtt = temp_dir.path().join("meeting.vtt");
    let explicit_output = temp_dir.path().join("custom.md");
    
    // Create a simple VTT file
    fs::write(&input_vtt, SIMPLE_VTT).expect("Failed to write VTT file");
    
    // Create existing file at explicit output location
    fs::write(&explicit_output, "existing content").expect("Failed to write existing file");
    
    // Conversion with explicit output should fail (auto-increment only applies to derived paths)
    let output = Command::new(get_vtt_to_md_path())
        .arg(&input_vtt)
        .arg(&explicit_output)
        .output()
        .expect("Failed to execute vtt-to-md");
    
    assert!(!output.status.success(), "Should fail when explicit output exists");
}
```

### Success Criteria:

#### Automated Verification:
- [ ] All existing tests pass: `cargo test`
- [ ] New auto-increment tests pass: `cargo test test_auto_increment_filename`
- [ ] No-auto-increment flag test passes: `cargo test test_no_auto_increment_flag`
- [ ] Explicit output test passes: `cargo test test_explicit_output_skips_auto_increment`
- [ ] Code compiles without warnings: `cargo build --release`
- [ ] Linting passes (if applicable): `cargo clippy`

#### Manual Verification:
- [ ] Convert same file 3 times: creates `file.md`, `file (1).md`, `file (2).md`
- [ ] Using `--no-auto-increment` without `--force` fails when file exists
- [ ] Using `--no-auto-increment --force` overwrites existing file
- [ ] Specifying explicit output path doesn't use auto-increment
- [ ] Help text displays correctly: `cargo run -- --help`

---

## Phase 2: Initialize WiX Configuration

### Overview
Set up cargo-wix tooling and create the base MSI installer configuration without file association. This establishes the foundation for Phase 3.

### Changes Required:

#### 1. Install cargo-wix (local development)
**Command**: Install cargo-wix subcommand

```powershell
cargo install cargo-wix
```

Note: This is a one-time setup for local development. GitHub Actions will install this in the workflow.

#### 2. Initialize WiX project structure
**Command**: Generate initial WiX configuration

```powershell
cargo wix init
```

This creates:
- `wix/main.wxs` - Main WiX source file
- `wix/main.wxs.license` - License file reference
- Updates `Cargo.toml` with `[package.metadata.wix]` section

#### 3. Customize wix/main.wxs for vtt-to-md
**File**: `wix/main.wxs`
**Changes**: Update product name, description, and installer settings

After `cargo wix init`, edit the generated file to customize:

```xml
<?xml version='1.0' encoding='windows-1252'?>
<Wix xmlns='http://schemas.microsoft.com/wix/2006/wi'>
    <Product
        Id='*'
        Name='VTT to Markdown'
        UpgradeCode='PUT-GUID-HERE'
        Manufacturer='vtt-to-md'
        Language='1033'
        Codepage='1252'
        Version='$(var.Version)'>

        <Package
            Id='*'
            Keywords='Installer'
            Description='VTT to Markdown converter - convert WebVTT transcript files to readable Markdown'
            Manufacturer='vtt-to-md'
            InstallerVersion='450'
            Languages='1033'
            Compressed='yes'
            InstallScope='perUser'
            SummaryCodepage='1252'
        />

        <!-- Rest of the generated content... -->
```

Key customizations:
- `Name='VTT to Markdown'` - Product display name
- `Description='VTT to Markdown converter...'` - Installer description
- `InstallScope='perUser'` - Per-user installation (no admin required)

#### 4. Update .gitignore for WiX artifacts
**File**: `.gitignore`
**Changes**: Add WiX build output directories

```ignore
/target
/wix/*.wixobj
/wix/*.wixpdb
*.msi
```

#### 5. Test local MSI build
**Command**: Build MSI installer locally

```powershell
cargo wix --nocapture
```

Expected output:
- MSI file created in `target/wix/vtt-to-md-{version}-x86_64.msi`
- No build errors or warnings

### Success Criteria:

#### Automated Verification:
- [ ] Project compiles: `cargo build --release`
- [ ] WiX build succeeds: `cargo wix --nocapture`
- [ ] MSI file exists: `Test-Path target/wix/vtt-to-md-*.msi`

#### Manual Verification:
- [ ] MSI installer runs without errors (double-click to test)
- [ ] Application installs to `%LOCALAPPDATA%\Programs\vtt-to-md\`
- [ ] Executable appears in install directory: `vtt-to-md.exe`
- [ ] Application can be run from install location
- [ ] Uninstaller works correctly (via Windows Settings > Apps)
- [ ] After uninstall, install directory is removed

---

## Phase 3: Configure Optional File Association

### Overview
Add per-user .vtt file association to the WiX installer with an optional checkbox (default checked). When enabled, double-clicking a .vtt file will invoke `vtt-to-md.exe "%1"` which uses the auto-increment feature from Phase 1.

### Changes Required:

#### 1. Add ProgId and Extension configuration to wix/main.wxs
**File**: `wix/main.wxs`
**Changes**: Add file association elements inside the Component that installs the executable

Locate the `<Component>` element that installs `vtt-to-md.exe` (search for `<File Id='vtt_to_md'` or similar), and add the ProgId configuration as a child of that Component:

```xml
        <Component Id='MainExecutable' Guid='PUT-GUID-HERE'>
            <File
                Id='vtt_to_md'
                Name='vtt-to-md.exe'
                DiskId='1'
                Source='$(var.CargoTargetBinDir)\vtt-to-md.exe'
                KeyPath='yes'
            />
            
            <!-- File Association ProgId -->
            <ProgId Id='VttToMd.VttFile' Description='VTT Transcript File'>
                <Extension Id='vtt' ContentType='text/vtt'>
                    <Verb Id='open' Command='Convert to Markdown' Argument='"%1"' />
                </Extension>
            </ProgId>
        </Component>
```

Key elements:
- `ProgId Id='VttToMd.VttFile'` - Unique program identifier
- `Extension Id='vtt'` - Register .vtt file extension
- `Verb Id='open' Argument='"%1"'` - Pass file path to executable
- No `--force` needed because Phase 1 auto-increment handles collisions

#### 2. Make file association optional with Feature checkbox
**File**: `wix/main.wxs`
**Changes**: Wrap file association in optional Feature

The generated `wix/main.wxs` will have a `<Feature>` element. Modify it to nest file association as a sub-feature:

```xml
        <Feature
            Id='MainProgram'
            Title='VTT to Markdown'
            Description='Main application'
            Level='1'
            ConfigurableDirectory='APPLICATIONFOLDER'
            AllowAdvertise='no'
            Display='expand'
            Absent='disallow'>
            
            <!-- Main application component -->
            <ComponentRef Id='MainExecutable' />
            <!-- Other components... -->
            
            <!-- Optional File Association -->
            <Feature
                Id='FileAssociation'
                Title='File Association'
                Description='Associate .vtt files with VTT to Markdown (double-click to convert)'
                Level='1'
                AllowAdvertise='no'>
                <Condition Level='1'>1</Condition>
            </Feature>
        </Feature>
```

Notes:
- `Level='1'` with `Condition Level='1'` means "checked by default"
- User can uncheck during installation to skip file association
- `Display='expand'` shows the sub-feature checkbox

#### 3. Update Cargo.toml metadata for WiX
**File**: `Cargo.toml`
**Changes**: Ensure WiX metadata section exists (created by `cargo wix init`)

After `cargo wix init`, the file should have:

```toml
[package.metadata.wix]
upgrade-guid = "PUT-GUID-HERE"
path-guid = "PUT-GUID-HERE"
license = false
eula = false
```

Verify the GUIDs are present. If not, generate them using:

```powershell
# In PowerShell
[guid]::NewGuid()
```

### Success Criteria:

#### Automated Verification:
- [ ] WiX build succeeds with file association: `cargo wix --nocapture`
- [ ] MSI file created: `Test-Path target/wix/vtt-to-md-*.msi`
- [ ] No WiX compiler warnings related to file association

#### Manual Verification:
- [ ] Installer displays file association checkbox during installation
- [ ] File association checkbox is checked by default
- [ ] Installing with checkbox checked: double-clicking .vtt file converts it
- [ ] Output file uses auto-increment: `file.md`, `file (1).md`, etc.
- [ ] Installing with checkbox unchecked: .vtt files not associated
- [ ] Uninstaller removes file association when installed
- [ ] Registry entry exists: `HKCU\Software\Classes\.vtt` (when associated)
- [ ] After uninstall, .vtt files no longer associated with application

### Phase 3 Implementation Complete

**Completion Date**: November 18, 2025

**Implementation Summary**:
- Added ProgId configuration (`VttToMd.VttFile`) to the `binary0` component in `wix/main.wxs`
- Registered `.vtt` extension with `text/vtt` content type
- Created `Convert to Markdown` verb that passes file path as `"%1"` argument
- Added optional `FileAssociation` feature with `Level='1'` (checked by default)
- File association integrates with Phase 1 auto-increment feature

**Notes for Reviewers**:
- File association is per-user (no admin privileges required)
- Users can uncheck the file association during installation
- Double-clicking `.vtt` files will automatically create incremented output files
- WiX build verification requires WiX Toolset installation (tested in CI via Phase 4)

**Commit**: `ade1edb` - "Phase 3: Add optional .vtt file association to MSI installer"

---

## Phase 4: Create GitHub Actions Release Workflow

### Overview
Automate MSI builds with GitHub Actions, triggered by pushing git tags (e.g., `v1.0.0`). The workflow installs WiX Toolset, extracts version from tag, builds the MSI, and uploads it as a release artifact.

### Changes Required:

#### 1. Create GitHub Actions workflow directory
**Directory**: `.github/workflows/`

```powershell
New-Item -ItemType Directory -Force -Path .github/workflows
```

#### 2. Create release workflow file
**File**: `.github/workflows/release.yml`
**Changes**: Create new workflow for building and releasing MSI

```yaml
name: Release

on:
  push:
    tags:
      - 'v*.*.*'

jobs:
  build-msi:
    name: Build MSI Installer
    runs-on: windows-latest
    
    steps:
      - name: Checkout code
        uses: actions/checkout@v4
      
      - name: Extract version from tag
        shell: pwsh
        run: |
          $tag = $env:GITHUB_REF -replace 'refs/tags/v', ''
          echo "VERSION=$tag" >> $env:GITHUB_ENV
          echo "Building version $tag"
      
      - name: Install Rust
        uses: dtolnay/rust-toolchain@stable
      
      - name: Install WiX Toolset
        shell: pwsh
        run: |
          # Download WiX 3.11.2 binaries
          Invoke-WebRequest -Uri "https://github.com/wixtoolset/wix3/releases/download/wix3112rtm/wix311-binaries.zip" -OutFile wix.zip
          Expand-Archive -Path wix.zip -DestinationPath wix-toolset
          $wixPath = (Get-Item "wix-toolset").FullName
          echo "$wixPath" >> $env:GITHUB_PATH
          # Verify installation
          & "$wixPath\candle.exe" -?
      
      - name: Install cargo-wix
        run: cargo install cargo-wix
      
      - name: Update Cargo.toml version
        shell: pwsh
        run: |
          $version = $env:VERSION
          (Get-Content Cargo.toml) -replace 'version = "0.1.0"', "version = `"$version`"" | Set-Content Cargo.toml
          Get-Content Cargo.toml | Select-String "version"
      
      - name: Build MSI
        run: cargo wix --nocapture
      
      - name: Rename MSI with version
        shell: pwsh
        run: |
          $version = $env:VERSION
          $msiFile = Get-ChildItem -Path target/wix/*.msi | Select-Object -First 1
          $newName = "vtt-to-md-$version-x86_64.msi"
          Rename-Item -Path $msiFile.FullName -NewName $newName
          echo "MSI_PATH=target/wix/$newName" >> $env:GITHUB_ENV
      
      - name: Upload MSI artifact
        uses: actions/upload-artifact@v4
        with:
          name: vtt-to-md-msi
          path: ${{ env.MSI_PATH }}
      
      - name: Create GitHub Release
        uses: softprops/action-gh-release@v1
        with:
          files: ${{ env.MSI_PATH }}
          body: |
            ## VTT to Markdown ${{ env.VERSION }}
            
            Windows MSI installer for vtt-to-md.
            
            ### Installation
            Download the MSI file and run it. Optionally enable .vtt file association during installation.
            
            ### Changes
            See [CHANGELOG.md](https://github.com/${{ github.repository }}/blob/main/CHANGELOG.md) for details.
          draft: false
          prerelease: false
        env:
          GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}
```

Key aspects:
- Triggers on tags matching `v*.*.*` pattern (e.g., `v1.0.0`, `v2.1.3`)
- Extracts version from tag and updates `Cargo.toml` dynamically
- Installs WiX Toolset 3.11.2 from GitHub releases
- Builds MSI using `cargo wix`
- Uploads MSI as both artifact and release attachment

#### 3. Update .gitignore for GitHub directories
**File**: `.gitignore`
**Changes**: Ensure .github directory is tracked

Verify `.github/` is NOT in `.gitignore` (it should be tracked). The current `.gitignore` is:

```ignore
/target
/wix/*.wixobj
/wix/*.wixpdb
*.msi
```

This is correct - `.github/` will be tracked by default.

#### 4. Create initial git tag for testing
**Command**: Create and push a test tag

```powershell
# After all changes are committed
git tag v0.2.0
git push origin v0.2.0
```

This will trigger the workflow for the first time.

### Success Criteria:

#### Automated Verification:
- [ ] Workflow file is valid YAML: Use GitHub Actions validator or `yamllint`
- [ ] Workflow appears in GitHub Actions tab after push
- [ ] Workflow triggers on tag push: `git push origin v0.2.0`
- [ ] All workflow steps complete successfully (green checkmarks)
- [ ] MSI artifact is uploaded to workflow run
- [ ] GitHub release is created automatically
- [ ] MSI file is attached to release

#### Manual Verification:
- [ ] Download MSI from GitHub release page
- [ ] MSI installer runs without errors
- [ ] Installed application works correctly
- [ ] File association checkbox appears during install
- [ ] Version number in installer matches git tag (e.g., v0.2.0 → 0.2.0)
- [ ] Creating a new tag triggers a new release automatically

### Phase 4 Implementation Complete

**Completion Date**: November 18, 2025

**Implementation Summary**:
- Created `.github/workflows/release.yml` workflow file
- Workflow triggers on version tags matching `v*.*.*` pattern
- Installs WiX Toolset 3.11.2 and cargo-wix in CI environment
- Extracts version from git tag and dynamically updates `Cargo.toml`
- Builds MSI using `cargo wix --nocapture`
- Renames MSI with version: `vtt-to-md-{version}-x86_64.msi`
- Uploads MSI as workflow artifact and attaches to GitHub release
- Generates release notes with installation instructions

**Notes for Reviewers**:
- Workflow uses PowerShell commands compatible with Windows runner
- WiX Toolset downloaded from official GitHub releases (stable)
- Version injection ensures MSI version matches git tag
- Release notes reference CHANGELOG.md for detailed changes
- Workflow requires no secrets beyond default `GITHUB_TOKEN`

**Testing Requirements**:
- Push a version tag (e.g., `v0.2.0`) to trigger the workflow
- Verify workflow completes successfully in GitHub Actions
- Download and test the generated MSI installer
- Confirm file association feature works as expected

**Commit**: `b86f21e` - "Phase 4: Create GitHub Actions release workflow"

---

## Testing Strategy

### Unit Tests:
- Auto-increment filename logic (`find_available_path()`)
- Path collision detection with various filename patterns
- `--no-auto-increment` flag behavior

### Integration Tests:
- Full conversion flow with auto-increment: file.md → file (1).md → file (2).md
- Backwards compatibility with `--no-auto-increment --force`
- Explicit output path bypassing auto-increment

### Installer Tests:
- MSI installation on clean Windows system
- File association registration in Windows registry
- Double-click .vtt file creates incremented .md files
- Uninstallation cleanup (files, registry, Start Menu)

### GitHub Actions Tests:
- Workflow triggers correctly on tag push
- Version extraction from git tags
- MSI build completes successfully
- Release artifact upload to GitHub

### Manual Testing Checklist:
1. Install MSI with file association enabled
2. Create test.vtt file, double-click → creates test.md
3. Double-click test.vtt again → creates test (1).md
4. Double-click test.vtt again → creates test (2).md
5. Verify all three files exist with different content
6. Uninstall via Windows Settings
7. Verify .vtt files no longer open with vtt-to-md

## Performance Considerations

- **Auto-increment loop**: Limited to 9999 iterations to prevent infinite loops
- **File existence checks**: Each check is a filesystem operation; loop should exit early
- **Installer size**: WiX creates compressed MSI, typically 1-2 MB for Rust binaries
- **Installation time**: Per-user installs are fast (< 10 seconds)

## Migration Notes

### For Users:
- **Behavior change**: Default behavior now creates `file (1).md` instead of failing
- **Backwards compatibility**: Use `--no-auto-increment --force` for old overwrite behavior
- **File association**: Optional during install, can be enabled/disabled via reinstall

### For Developers:
- **Local builds**: Install cargo-wix and WiX Toolset for local MSI testing
- **Version management**: Update version in git tag; Cargo.toml updated automatically in CI
- **Release process**: Push git tag to trigger automated MSI build and release

## References

- Original Issue: https://github.com/lossyrob/vtt-to-md/issues/13
- Spec: `.paw/work/msi-installer-file-association/Spec.md` (not created)
- Research: `.paw/work/msi-installer-file-association/CodeResearch.md`
- cargo-wix documentation: https://volks73.github.io/cargo-wix/
- WiX Toolset file association: https://www.advancedinstaller.com/versus/wix-toolset/register-file-association-with-wix.html
- GitHub Actions workflow syntax: https://docs.github.com/en/actions/reference/workflow-syntax-for-github-actions
