# MSI Installer with File Association

## Overview

This enhancement adds Windows MSI installer support for vtt-to-md with optional .vtt file association. The installer enables one-click installation and allows users to convert VTT files by double-clicking them in Windows Explorer.

**Key capabilities:**
- Per-user MSI installer (no admin privileges required)
- Optional .vtt file association (checkbox during installation, default checked)
- Auto-increment filename collision handling (browser-style)
- Automated GitHub Actions release workflow triggered by git tags

**Problem solved:** Previously, users needed to manually install via cargo or build from source. File conversion required command-line invocation. This installer provides a native Windows installation experience with Explorer integration.

## Architecture and Design

### Components

1. **Auto-increment collision handling** (`src/cli.rs`):
   - New default behavior: `file.md` → `file (1).md` → `file (2).md`
   - Replaces "file exists" error with automatic filename increment
   - Provides `--no-auto-increment` flag for backwards compatibility

2. **WiX installer configuration** (`wix/main.wxs`):
   - Per-user installation to `%LOCALAPPDATA%\vtt-to-md`
   - Optional PATH environment variable addition
   - Optional `.vtt` file association via ProgId/Extension elements
   - Uses WiX Toolset 3.11.2 for MSI generation

3. **GitHub Actions workflow** (`.github/workflows/release.yml`):
   - Triggers on version tags (e.g., `v1.0.0`)
   - Builds MSI with cargo-wix
   - Uploads installer to GitHub Releases
   - Automatic version injection from git tag

### Design Decisions

**Why auto-increment as default?**
File association invokes the app with absolute paths. Without auto-increment, users would encounter errors when converting the same file multiple times. Browser-style incrementation provides familiar UX.

**Why per-user installation?**
- No admin privileges required
- Simpler installation experience
- Appropriate scope for single-user developer tool

**Why optional file association?**
Respects user choice. Some users may prefer explicit CLI invocation or have existing .vtt handlers.

**Why WiX over other tools?**
cargo-wix provides Rust-native integration. WiX is the standard Windows installer framework with excellent documentation and GitHub Actions support.

## Installation

### For End Users

1. Download MSI from [GitHub Releases](https://github.com/lossyrob/vtt-to-md/releases)
2. Run the installer (double-click MSI file)
3. During installation:
   - Check "PATH Environment Variable" to add `vtt-to-md` to PATH (optional)
   - Check "File Association" to associate .vtt files (optional, default checked)
4. Complete installation

**Installation location:** `%LOCALAPPDATA%\vtt-to-md\bin\vtt-to-md.exe`

### For Developers

Build MSI locally:
```powershell
# Install cargo-wix (one-time setup)
cargo install cargo-wix

# Build MSI
cargo wix --nocapture
```

Output: `target/wix/vtt-to-md-{version}-x86_64.msi`

## File Association Behavior

When file association is enabled during installation:

1. **Double-click .vtt file** → Opens with vtt-to-md
2. **First conversion** creates `meeting.md` in same directory
3. **Second conversion** creates `meeting (1).md`
4. **Subsequent conversions** create `meeting (2).md`, `meeting (3).md`, etc.

**Auto-increment algorithm:**
- Checks if output file exists
- Tries numbered suffixes: `(1)`, `(2)`, ..., `(9999)`
- Returns first available filename
- No confirmation prompts or overwrites

**Disabling auto-increment:**
Use `--no-auto-increment` flag for old behavior (requires `--force` to overwrite):
```bash
vtt-to-md --no-auto-increment --force input.vtt
```

## Testing Guide

### Testing Auto-Increment Feature

**Command-line test:**
```powershell
# Create test VTT file
echo "WEBVTT`n`n00:00:00.000 --> 00:00:05.000`n<v Alice>Hello world" > test.vtt

# First conversion
vtt-to-md test.vtt
# Creates: test.md

# Second conversion
vtt-to-md test.vtt
# Creates: test (1).md

# Third conversion
vtt-to-md test.vtt
# Creates: test (2).md

# Verify all files exist
dir test*.md
```

**Expected:** Three distinct files without errors.

**Automated tests:**
```powershell
cargo test test_auto_increment_filename
cargo test test_no_auto_increment_flag
cargo test test_explicit_output_skips_auto_increment
```

### Testing MSI Installation

**Prerequisites:** Windows 10/11 x64

**Test procedure:**
1. Build MSI: `cargo wix --nocapture`
2. Install MSI from `target/wix/`
3. Verify installation:
   ```powershell
   Test-Path "$env:LOCALAPPDATA\vtt-to-md\bin\vtt-to-md.exe"
   ```
4. Test command execution:
   ```powershell
   & "$env:LOCALAPPDATA\vtt-to-md\bin\vtt-to-md.exe" --version
   ```

### Testing File Association

**Prerequisites:** MSI installed with "File Association" checked

**Test procedure:**
1. Create `test.vtt` file with sample content
2. Right-click → "Open with" → Should show "VTT to Markdown"
3. Double-click `test.vtt`
4. Verify `test.md` created in same directory
5. Double-click `test.vtt` again
6. Verify `test (1).md` created
7. Check registry:
   ```powershell
   Get-Item -Path "HKCU:\Software\Classes\.vtt"
   ```

**Expected:**
- `.vtt` files show custom icon (if configured)
- Double-click triggers conversion without errors
- Incremented files appear without overwrites

### Testing GitHub Actions Workflow

**Trigger workflow:**
```powershell
git tag v0.2.0
git push origin v0.2.0
```

**Verify in GitHub:**
1. Navigate to Actions tab
2. Confirm "Release" workflow triggered
3. Check all steps complete successfully
4. Verify MSI uploaded to Releases
5. Download and test MSI from release page

**Automated:** Workflow includes WiX build verification step.

## Edge Cases and Limitations

### Auto-Increment
- **Limit:** 9999 increments maximum (extremely unlikely to hit)
- **Behavior at limit:** Falls back to base filename (will fail during write)
- **Explicit output:** Auto-increment only applies to derived output paths, not explicit `--output` arguments

### File Association
- **Per-user scope:** Association only applies to current user
- **Uninstall:** Association removed automatically
- **Conflicts:** If another program registers .vtt, last installer wins
- **Double-click limitation:** No CLI flags supported (uses default behavior with auto-increment)

### MSI Installer
- **Platform:** Windows x64 only (no x86 or ARM builds)
- **Upgrade:** Major upgrades supported via WiX UpgradeCode
- **Downgrade:** Not supported (installer shows error)
- **Silent install:** Supported via `msiexec /i installer.msi /qn`

### GitHub Actions
- **Version source:** Git tag only (Cargo.toml dynamically updated)
- **Workflow trigger:** Manual tag push required
- **Build time:** ~5-10 minutes including Rust and WiX setup
- **Artifacts:** MSI attached to release, also available as workflow artifact

## Migration and Compatibility

### For Existing Users

**Behavior change:** Default behavior now creates incremented files instead of failing:
- **Old:** `vtt-to-md test.vtt` (when test.md exists) → Error
- **New:** `vtt-to-md test.vtt` → Creates `test (1).md`

**Backwards compatibility:** Use `--no-auto-increment --force` to overwrite existing files.

**Upgrading:** Installer supports major upgrades. Install new MSI over old version.

### For Developers

**New development workflow:**
1. Implement features on feature branch
2. Merge to main via PR
3. Create version tag: `git tag v1.0.0 && git push origin v1.0.0`
4. GitHub Actions automatically builds and releases MSI

**Local MSI testing:**
```powershell
# Install cargo-wix (one-time)
cargo install cargo-wix

# Build MSI
cargo wix --nocapture

# Test locally
msiexec /i target/wix/vtt-to-md-*.msi /l*v install.log
```

## References

- Implementation Plan: `.paw/work/msi-installer-file-association/ImplementationPlan.md`
- Original Issue: https://github.com/lossyrob/vtt-to-md/issues/13
- cargo-wix documentation: https://volks73.github.io/cargo-wix/
- WiX Toolset: https://wixtoolset.org/documentation/
- GitHub Actions: `.github/workflows/release.yml`

## Related Files

- Auto-increment implementation: `src/cli.rs` (lines 150-185)
- WiX configuration: `wix/main.wxs`
- Integration tests: `tests/integration_test.rs` (lines 503-600)
- Release workflow: `.github/workflows/release.yml`
