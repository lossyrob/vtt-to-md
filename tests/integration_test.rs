//! Integration tests for the vtt-to-md CLI tool.
//!
//! These tests validate the end-to-end functionality of the tool by executing
//! the compiled binary with various inputs and flags, verifying both successful
//! conversions and appropriate error handling.
//!
//! Test coverage includes:
//! - Basic conversion functionality
//! - All CLI flags (--force, --no-clobber, --stdout, --unknown-speaker, --include-timestamps)
//! - Error conditions with correct exit codes (66, 65, 73)
//! - Path handling (spaces, custom output)
//! - Speaker consolidation

use std::fs;
use std::path::PathBuf;
use std::process::Command;
use tempfile::TempDir;

/// Get the path to the compiled vtt-to-md executable for testing.
///
/// This function locates the test binary in the target directory,
/// adjusting for platform-specific executable names (.exe on Windows).
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

fn create_test_vtt(dir: &TempDir, filename: &str, content: &str) -> PathBuf {
    let path = dir.path().join(filename);
    fs::write(&path, content).expect("Failed to write test VTT file");
    path
}

fn new_test_command(vtt_to_md: &PathBuf) -> (Command, TempDir) {
    let config_root = TempDir::new().expect("Failed to create temp config directory");
    let mut cmd = Command::new(vtt_to_md);

    // Ensure tests are not affected by user environment/config.
    cmd.env_remove("VTT_TO_MD_INCLUDE_TIMESTAMPS");

    // Point the per-user config directory at a temp folder.
    #[cfg(target_os = "windows")]
    cmd.env("APPDATA", config_root.path());
    #[cfg(target_os = "linux")]
    cmd.env("XDG_CONFIG_HOME", config_root.path());
    #[cfg(target_os = "macos")]
    cmd.env("HOME", config_root.path());

    (cmd, config_root)
}

fn test_config_file_path(config_root: &TempDir) -> PathBuf {
    #[cfg(target_os = "macos")]
    {
        return config_root
            .path()
            .join("Library")
            .join("Application Support")
            .join("vtt-to-md")
            .join("config.toml");
    }

    #[cfg(not(target_os = "macos"))]
    {
        return config_root
            .path()
            .join("vtt-to-md")
            .join("config.toml");
    }
}

fn write_test_config(config_root: &TempDir, contents: &str) -> PathBuf {
    let path = test_config_file_path(config_root);
    let parent = path
        .parent()
        .expect("Config file path should have a parent directory");
    fs::create_dir_all(parent).expect("Failed to create config directory");
    fs::write(&path, contents).expect("Failed to write config file");
    path
}

const SIMPLE_VTT: &str = "WEBVTT\n\n00:00:00.000 --> 00:00:02.000\n<v Alice>Hello world</v>\n";

#[test]
fn test_basic_conversion() {
    let temp_dir = TempDir::new().unwrap();
    let vtt_content = "WEBVTT\n\n00:00:00.000 --> 00:00:02.000\n<v Alice>Hello world</v>\n\n00:00:02.000 --> 00:00:04.000\n<v Bob>Hi there</v>\n";
    let input_path = create_test_vtt(&temp_dir, "test.vtt", vtt_content);
    let output_path = temp_dir.path().join("test.md");

    let vtt_to_md = get_vtt_to_md_path();
    let (mut cmd, _config_root) = new_test_command(&vtt_to_md);
    let output = cmd
        .arg(input_path.to_str().unwrap())
        .output()
        .expect("Failed to execute vtt-to-md");

    assert!(
        output.status.success(),
        "Command failed: {:?}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(output_path.exists(), "Output file was not created");

    let markdown = fs::read_to_string(&output_path).expect("Failed to read output file");
    assert!(markdown.contains("**Alice:** Hello world"));
    assert!(markdown.contains("**Bob:** Hi there"));
}

#[test]
fn test_force_flag_overwrites() {
    let temp_dir = TempDir::new().unwrap();
    let vtt_content = "WEBVTT\n\n00:00:00.000 --> 00:00:02.000\n<v Alice>Hello</v>\n";
    let input_path = create_test_vtt(&temp_dir, "test.vtt", vtt_content);
    let output_path = temp_dir.path().join("test.md");

    // Create output file first
    fs::write(&output_path, "existing content").expect("Failed to create existing file");

    let vtt_to_md = get_vtt_to_md_path();
    let (mut cmd, _config_root) = new_test_command(&vtt_to_md);
    let output = cmd
        .arg(input_path.to_str().unwrap())
        .arg("--no-auto-increment")
        .arg("--force")
        .output()
        .expect("Failed to execute vtt-to-md");

    assert!(
        output.status.success(),
        "Command failed with --force: {:?}",
        String::from_utf8_lossy(&output.stderr)
    );

    let markdown = fs::read_to_string(&output_path).expect("Failed to read output file");
    assert!(markdown.contains("**Alice:** Hello"));
    assert!(!markdown.contains("existing content"));
}

#[test]
fn test_no_clobber_flag_skips() {
    let temp_dir = TempDir::new().unwrap();
    let vtt_content = "WEBVTT\n\n00:00:00.000 --> 00:00:02.000\n<v Alice>Hello</v>\n";
    let input_path = create_test_vtt(&temp_dir, "test.vtt", vtt_content);
    let output_path = temp_dir.path().join("test.md");

    // Create output file first
    fs::write(&output_path, "existing content").expect("Failed to create existing file");

    let vtt_to_md = get_vtt_to_md_path();
    let (mut cmd, _config_root) = new_test_command(&vtt_to_md);
    let output = cmd
        .arg(input_path.to_str().unwrap())
        .arg("--no-clobber")
        .output()
        .expect("Failed to execute vtt-to-md");

    assert!(
        output.status.success(),
        "Command should succeed with --no-clobber"
    );

    let content = fs::read_to_string(&output_path).expect("Failed to read output file");
    assert_eq!(content, "existing content", "File should not be modified");
}

#[test]
fn test_stdout_flag() {
    let temp_dir = TempDir::new().unwrap();
    let vtt_content = "WEBVTT\n\n00:00:00.000 --> 00:00:02.000\n<v Alice>Hello</v>\n";
    let input_path = create_test_vtt(&temp_dir, "test.vtt", vtt_content);
    let output_path = temp_dir.path().join("test.md");

    let vtt_to_md = get_vtt_to_md_path();
    let (mut cmd, _config_root) = new_test_command(&vtt_to_md);
    let output = cmd
        .arg(input_path.to_str().unwrap())
        .arg("--stdout")
        .output()
        .expect("Failed to execute vtt-to-md");

    assert!(output.status.success(), "Command failed with --stdout");
    assert!(
        !output_path.exists(),
        "Output file should not be created with --stdout"
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("**Alice:** Hello"));
}

#[test]
fn test_unknown_speaker_flag() {
    let temp_dir = TempDir::new().unwrap();
    let vtt_content = "WEBVTT\n\n00:00:00.000 --> 00:00:02.000\nText without speaker\n";
    let input_path = create_test_vtt(&temp_dir, "test.vtt", vtt_content);

    let vtt_to_md = get_vtt_to_md_path();
    let (mut cmd, _config_root) = new_test_command(&vtt_to_md);
    let output = cmd
        .arg(input_path.to_str().unwrap())
        .arg("--unknown-speaker")
        .arg("Narrator")
        .arg("--stdout")
        .output()
        .expect("Failed to execute vtt-to-md");

    assert!(
        output.status.success(),
        "Command failed with --unknown-speaker"
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("**Narrator:**"));
}

#[test]
fn test_include_timestamps_flag_enables_timestamps() {
    let temp_dir = TempDir::new().unwrap();
    let vtt_content = "WEBVTT\n\n00:00:00.000 --> 00:00:02.000\n<v Alice>Hello</v>\n\n00:00:02.000 --> 00:00:04.000\n<v Alice>World</v>\n";
    let input_path = create_test_vtt(&temp_dir, "test.vtt", vtt_content);

    let vtt_to_md = get_vtt_to_md_path();
    let (mut cmd, _config_root) = new_test_command(&vtt_to_md);
    let output = cmd
        .arg(input_path.to_str().unwrap())
        .arg("--include-timestamps")
        .arg("--stdout")
        .output()
        .expect("Failed to execute vtt-to-md");

    assert!(
        output.status.success(),
        "Command failed with --include-timestamps"
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("[00:00:00.000]"));
    assert!(stdout.contains("**Alice:**"));
}

#[test]
fn test_include_timestamps_explicit_false_disables_timestamps() {
    let temp_dir = TempDir::new().unwrap();
    let vtt_content = "WEBVTT\n\n00:00:00.000 --> 00:00:02.000\n<v Alice>Hello</v>\n\n00:00:02.000 --> 00:00:04.000\n<v Alice>World</v>\n";
    let input_path = create_test_vtt(&temp_dir, "test.vtt", vtt_content);

    let vtt_to_md = get_vtt_to_md_path();
    let (mut cmd, _config_root) = new_test_command(&vtt_to_md);
    let output = cmd
        .arg(input_path.to_str().unwrap())
        .arg("--include-timestamps=false")
        .arg("--stdout")
        .output()
        .expect("Failed to execute vtt-to-md");

    assert!(
        output.status.success(),
        "Command failed with --include-timestamps=false"
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(!stdout.contains("[00:00:00.000]"));
}

#[test]
fn test_include_timestamps_legacy_cli_value_is_accepted_with_warning() {
    let temp_dir = TempDir::new().unwrap();
    let vtt_content = "WEBVTT\n\n00:00:00.000 --> 00:00:02.000\n<v Alice>Hello</v>\n";
    let input_path = create_test_vtt(&temp_dir, "test.vtt", vtt_content);

    let vtt_to_md = get_vtt_to_md_path();
    let (mut cmd, _config_root) = new_test_command(&vtt_to_md);
    let output = cmd
        .arg(input_path.to_str().unwrap())
        .arg("--include-timestamps")
        .arg("first")
        .arg("--stdout")
        .output()
        .expect("Failed to execute vtt-to-md");

    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("[00:00:00.000]"));

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("Warning:"));
    assert!(stderr.contains("legacy"));
}

#[test]
fn test_file_not_found_error() {
    let vtt_to_md = get_vtt_to_md_path();
    let (mut cmd, _config_root) = new_test_command(&vtt_to_md);
    let output = cmd
        .arg("nonexistent.vtt")
        .output()
        .expect("Failed to execute vtt-to-md");

    assert!(
        !output.status.success(),
        "Command should fail for nonexistent file"
    );
    assert_eq!(
        output.status.code(),
        Some(66),
        "Exit code should be 66 (EX_NOINPUT)"
    );
}

#[test]
fn test_invalid_vtt_format() {
    let temp_dir = TempDir::new().unwrap();
    let invalid_content = "NOT A WEBVTT FILE\n\nSome random text\n";
    let input_path = create_test_vtt(&temp_dir, "invalid.vtt", invalid_content);

    let vtt_to_md = get_vtt_to_md_path();
    let (mut cmd, _config_root) = new_test_command(&vtt_to_md);
    let output = cmd
        .arg(input_path.to_str().unwrap())
        .arg("--stdout")
        .output()
        .expect("Failed to execute vtt-to-md");

    assert!(
        !output.status.success(),
        "Command should fail for invalid VTT format"
    );
    assert_eq!(
        output.status.code(),
        Some(65),
        "Exit code should be 65 (EX_DATAERR)"
    );
}

#[test]
fn test_output_exists_without_force() {
    let temp_dir = TempDir::new().unwrap();
    let vtt_content = "WEBVTT\n\n00:00:00.000 --> 00:00:02.000\n<v Alice>Hello</v>\n";
    let input_path = create_test_vtt(&temp_dir, "test.vtt", vtt_content);
    let output_path = temp_dir.path().join("test.md");

    // Create output file first
    fs::write(&output_path, "existing content").expect("Failed to create existing file");

    let vtt_to_md = get_vtt_to_md_path();
    let (mut cmd, _config_root) = new_test_command(&vtt_to_md);
    let output = cmd
        .arg(input_path.to_str().unwrap())
        .arg("--no-auto-increment")
        .output()
        .expect("Failed to execute vtt-to-md");

    assert!(
        !output.status.success(),
        "Command should fail when output exists without --force"
    );
    assert_eq!(
        output.status.code(),
        Some(73),
        "Exit code should be 73 (EX_CANTCREAT)"
    );

    let content = fs::read_to_string(&output_path).expect("Failed to read output file");
    assert_eq!(content, "existing content", "File should not be modified");
}

#[test]
fn test_consolidate_consecutive_speakers() {
    let temp_dir = TempDir::new().unwrap();
    let vtt_content = "WEBVTT\n\n00:00:00.000 --> 00:00:02.000\n<v Alice>Hello</v>\n\n00:00:02.000 --> 00:00:04.000\n<v Alice>world</v>\n\n00:00:04.000 --> 00:00:06.000\n<v Bob>Hi there</v>\n";
    let input_path = create_test_vtt(&temp_dir, "test.vtt", vtt_content);

    let vtt_to_md = get_vtt_to_md_path();
    let (mut cmd, _config_root) = new_test_command(&vtt_to_md);
    let output = cmd
        .arg(input_path.to_str().unwrap())
        .arg("--stdout")
        .output()
        .expect("Failed to execute vtt-to-md");

    assert!(output.status.success(), "Command failed");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("**Alice:** Hello world"));
    assert!(stdout.contains("**Bob:** Hi there"));
}

#[test]
fn test_path_with_spaces() {
    let temp_dir = TempDir::new().unwrap();
    let vtt_content = "WEBVTT\n\n00:00:00.000 --> 00:00:02.000\n<v Alice>Hello</v>\n";
    let input_path = create_test_vtt(&temp_dir, "test file.vtt", vtt_content);
    let output_path = temp_dir.path().join("test file.md");

    let vtt_to_md = get_vtt_to_md_path();
    let (mut cmd, _config_root) = new_test_command(&vtt_to_md);
    let output = cmd
        .arg(input_path.to_str().unwrap())
        .output()
        .expect("Failed to execute vtt-to-md");

    assert!(
        output.status.success(),
        "Command failed with path containing spaces: {:?}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(output_path.exists(), "Output file was not created");

    let markdown = fs::read_to_string(&output_path).expect("Failed to read output file");
    assert!(markdown.contains("**Alice:** Hello"));
}

#[test]
fn test_custom_output_path() {
    let temp_dir = TempDir::new().unwrap();
    let vtt_content = "WEBVTT\n\n00:00:00.000 --> 00:00:02.000\n<v Alice>Hello</v>\n";
    let input_path = create_test_vtt(&temp_dir, "test.vtt", vtt_content);
    let output_path = temp_dir.path().join("custom_output.md");

    let vtt_to_md = get_vtt_to_md_path();
    let (mut cmd, _config_root) = new_test_command(&vtt_to_md);
    let output = cmd
        .arg(input_path.to_str().unwrap())
        .arg(output_path.to_str().unwrap())
        .output()
        .expect("Failed to execute vtt-to-md");

    assert!(
        output.status.success(),
        "Command failed with custom output path"
    );
    assert!(output_path.exists(), "Custom output file was not created");

    let markdown = fs::read_to_string(&output_path).expect("Failed to read output file");
    assert!(markdown.contains("**Alice:** Hello"));
}

#[test]
fn test_multiline_voice_tags() {
    let temp_dir = TempDir::new().unwrap();
    // Test VTT with multi-line content within a single voice tag
    // This mimics the Teams format where text can span multiple lines within one cue
    let vtt_content = "WEBVTT\n\n\
13c7246d-4823-4d01-9e30-b633355ec6bb/31-0\n\
00:00:14.458 --> 00:00:18.893\n\
<v Speaker1>And and that we can go.\n\
But imagine for the user experience being</v>\n\
\n\
13c7246d-4823-4d01-9e30-b633355ec6bb/31-1\n\
00:00:18.893 --> 00:00:24.335\n\
<v Speaker1>you have a prompt that blue like kind of\n\
truly brings up entire sample app life,</v>\n\
\n\
13c7246d-4823-4d01-9e30-b633355ec6bb/31-2\n\
00:00:24.335 --> 00:00:24.738\n\
<v Speaker1>right?</v>\n";
    let input_path = create_test_vtt(&temp_dir, "multiline.vtt", vtt_content);

    let vtt_to_md = get_vtt_to_md_path();
    let (mut cmd, _config_root) = new_test_command(&vtt_to_md);
    let output = cmd
        .arg(input_path.to_str().unwrap())
        .arg("--stdout")
        .output()
        .expect("Failed to execute vtt-to-md");

    assert!(
        output.status.success(),
        "Command failed with multi-line voice tags: {:?}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    
    // The key test: all three parts should be consolidated into one paragraph
    // and the text should include ALL content from all three cues
    assert!(stdout.contains("**Speaker1:**"), "Speaker should be present");
    assert!(
        stdout.contains("But imagine for the user experience being"),
        "Multi-line content from first cue should be preserved"
    );
    assert!(
        stdout.contains("you have a prompt"),
        "Content from second cue should be present"
    );
    assert!(
        stdout.contains("truly brings up entire sample app life"),
        "Multi-line content from second cue should be preserved"
    );
    assert!(stdout.contains("right?"), "Content from third cue should be present");
    
    // Verify it's all in one paragraph (no extra ** for same speaker)
    let speaker_count = stdout.matches("**Speaker1:**").count();
    assert_eq!(
        speaker_count, 1,
        "All consecutive cues from same speaker should be consolidated into one paragraph"
    );
}

#[test]
fn test_filter_unknown_flag() {
    let temp_dir = TempDir::new().unwrap();
    // Test VTT with speaker cues and cues without speakers (unknown)
    let vtt_content = "WEBVTT\n\n\
00:00:00.000 --> 00:00:02.000\n\
<v Alice>Hello world</v>\n\
\n\
00:00:02.000 --> 00:00:03.000\n\
Umm.\n\
\n\
00:00:03.000 --> 00:00:05.000\n\
<v Alice>How are you?</v>\n\
\n\
00:00:05.000 --> 00:00:06.000\n\
Yeah.\n\
\n\
00:00:06.000 --> 00:00:08.000\n\
<v Bob>I'm fine, thanks!</v>\n";
    let input_path = create_test_vtt(&temp_dir, "test.vtt", vtt_content);

    let vtt_to_md = get_vtt_to_md_path();
    
    // Test without flags - Teams format auto-detected, so Unknown speakers should be filtered
    let (mut cmd, _config_root) = new_test_command(&vtt_to_md);
    let output = cmd
        .arg(input_path.to_str().unwrap())
        .arg("--stdout")
        .output()
        .expect("Failed to execute vtt-to-md");

    assert!(output.status.success(), "Command failed without flags");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(!stdout.contains("**Unknown:**"), "Teams format auto-filters Unknown speakers");
    assert!(!stdout.contains("Umm."), "Should not contain unknown cue text");
    
    // Test with --no-filter-unknown - should include Unknown speakers
    let (mut cmd, _config_root) = new_test_command(&vtt_to_md);
    let output = cmd
        .arg(input_path.to_str().unwrap())
        .arg("--no-filter-unknown")
        .arg("--stdout")
        .output()
        .expect("Failed to execute vtt-to-md with --no-filter-unknown");

    assert!(
        output.status.success(),
        "Command failed with --no-filter-unknown"
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("**Unknown:**"), "Should contain Unknown speaker when not filtered");
    assert!(stdout.contains("Umm."), "Should contain unknown cue text");
    assert!(stdout.contains("Yeah."), "Should contain unknown cue text");
    
    // Test with explicit --filter-unknown flag - should also exclude Unknown speakers
    let (mut cmd, _config_root) = new_test_command(&vtt_to_md);
    let output = cmd
        .arg(input_path.to_str().unwrap())
        .arg("--filter-unknown")
        .arg("--stdout")
        .output()
        .expect("Failed to execute vtt-to-md with --filter-unknown");

    assert!(
        output.status.success(),
        "Command failed with --filter-unknown"
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(!stdout.contains("**Unknown:**"), "Should not contain Unknown speaker");
    assert!(!stdout.contains("Umm."), "Should not contain unknown cue text");
    assert!(!stdout.contains("Yeah."), "Should not contain unknown cue text");
    
    // Verify Alice and Bob are still present and consolidated (in filtered output)
    assert!(stdout.contains("**Alice:** Hello world How are you?"), "Alice's cues should be consolidated");
    assert!(stdout.contains("**Bob:** I'm fine, thanks!"), "Bob should be present");
}

#[test]
fn test_auto_increment_filename() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let input_vtt = temp_dir.path().join("meeting.vtt");
    
    // Create a simple VTT file
    fs::write(&input_vtt, SIMPLE_VTT).expect("Failed to write VTT file");
    
    // First conversion: creates meeting.md
    let vtt_to_md = get_vtt_to_md_path();
    let (mut cmd, _config_root) = new_test_command(&vtt_to_md);
    let output = cmd
        .arg(&input_vtt)
        .output()
        .expect("Failed to execute vtt-to-md");
    
    assert!(output.status.success(), "First conversion failed");
    
    let first_output = temp_dir.path().join("meeting.md");
    assert!(first_output.exists(), "meeting.md should exist");
    
    // Second conversion: should create meeting (1).md
    let (mut cmd, _config_root) = new_test_command(&vtt_to_md);
    let output = cmd
        .arg(&input_vtt)
        .output()
        .expect("Failed to execute vtt-to-md");
    
    assert!(output.status.success(), "Second conversion failed");
    
    let second_output = temp_dir.path().join("meeting (1).md");
    assert!(second_output.exists(), "meeting (1).md should exist");
    
    // Third conversion: should create meeting (2).md
    let (mut cmd, _config_root) = new_test_command(&vtt_to_md);
    let output = cmd
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
    let vtt_to_md = get_vtt_to_md_path();
    let (mut cmd, _config_root) = new_test_command(&vtt_to_md);
    let output = cmd
        .arg(&input_vtt)
        .output()
        .expect("Failed to execute vtt-to-md");
    
    assert!(output.status.success());
    assert!(output_md.exists());
    
    // Second conversion with --no-auto-increment should fail
    let (mut cmd, _config_root) = new_test_command(&vtt_to_md);
    let output = cmd
        .arg("--no-auto-increment")
        .arg(&input_vtt)
        .output()
        .expect("Failed to execute vtt-to-md");
    
    assert!(!output.status.success(), "Should fail when output exists");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("already exists") || stderr.contains("OutputExists"));
    
    // Should succeed with --force
    let (mut cmd, _config_root) = new_test_command(&vtt_to_md);
    let output = cmd
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
    let vtt_to_md = get_vtt_to_md_path();
    let (mut cmd, _config_root) = new_test_command(&vtt_to_md);
    let output = cmd
        .arg(&input_vtt)
        .arg(&explicit_output)
        .output()
        .expect("Failed to execute vtt-to-md");

    assert!(!output.status.success(), "Should fail when explicit output exists");
}

#[cfg(target_os = "windows")]
mod include_timestamps_defaults_windows {
    use super::{create_test_vtt, new_test_command, write_test_config};
    use super::get_vtt_to_md_path;
    use tempfile::TempDir;

    #[test]
    fn env_var_default_is_used_when_flag_omitted() {
        let temp_dir = TempDir::new().unwrap();
        let input_path = create_test_vtt(
            &temp_dir,
            "test.vtt",
            "WEBVTT\n\n00:00:00.000 --> 00:00:02.000\n<v Alice>Hello</v>\n",
        );

        let vtt_to_md = get_vtt_to_md_path();
        let (mut cmd, _config_root) = new_test_command(&vtt_to_md);
        let output = cmd
            .arg(input_path.to_str().unwrap())
            .arg("--stdout")
            .env("VTT_TO_MD_INCLUDE_TIMESTAMPS", "true")
            .output()
            .expect("Failed to execute vtt-to-md");

        assert!(output.status.success());
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("[00:00:00.000]"));
    }

    #[test]
    fn config_default_is_used_when_no_env_and_flag_omitted() {
        let temp_dir = TempDir::new().unwrap();
        let input_path = create_test_vtt(
            &temp_dir,
            "test.vtt",
            "WEBVTT\n\n00:00:00.000 --> 00:00:02.000\n<v Alice>Hello</v>\n",
        );

        let vtt_to_md = get_vtt_to_md_path();
        let (mut cmd, config_root) = new_test_command(&vtt_to_md);
        write_test_config(&config_root, "include_timestamps = true\n");

        let output = cmd
            .arg(input_path.to_str().unwrap())
            .arg("--stdout")
            .output()
            .expect("Failed to execute vtt-to-md");

        assert!(output.status.success());
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("[00:00:00.000]"));
    }

    #[test]
    fn cli_overrides_env_var_default() {
        let temp_dir = TempDir::new().unwrap();
        let input_path = create_test_vtt(
            &temp_dir,
            "test.vtt",
            "WEBVTT\n\n00:00:00.000 --> 00:00:02.000\n<v Alice>Hello</v>\n",
        );

        let vtt_to_md = get_vtt_to_md_path();
        let (mut cmd, _config_root) = new_test_command(&vtt_to_md);
        let output = cmd
            .arg(input_path.to_str().unwrap())
            .arg("--include-timestamps=false")
            .arg("--stdout")
            .env("VTT_TO_MD_INCLUDE_TIMESTAMPS", "true")
            .output()
            .expect("Failed to execute vtt-to-md");

        assert!(output.status.success());
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(!stdout.contains("[00:00:00.000]"));
    }

    #[test]
    fn invalid_env_var_falls_back_to_config_with_warning() {
        let temp_dir = TempDir::new().unwrap();
        let input_path = create_test_vtt(
            &temp_dir,
            "test.vtt",
            "WEBVTT\n\n00:00:00.000 --> 00:00:02.000\n<v Alice>Hello</v>\n",
        );

        let vtt_to_md = get_vtt_to_md_path();
        let (mut cmd, config_root) = new_test_command(&vtt_to_md);
        write_test_config(&config_root, "include_timestamps = true\n");

        let output = cmd
            .arg(input_path.to_str().unwrap())
            .arg("--stdout")
            .env("VTT_TO_MD_INCLUDE_TIMESTAMPS", "bogus")
            .output()
            .expect("Failed to execute vtt-to-md");

        assert!(output.status.success());
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("[00:00:00.000]"));

        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(stderr.contains("Warning:"));
        assert!(stderr.contains("VTT_TO_MD_INCLUDE_TIMESTAMPS"));
    }

    #[test]
    fn invalid_config_falls_back_to_built_in_default_with_warning() {
        let temp_dir = TempDir::new().unwrap();
        let input_path = create_test_vtt(
            &temp_dir,
            "test.vtt",
            "WEBVTT\n\n00:00:00.000 --> 00:00:02.000\n<v Alice>Hello</v>\n",
        );

        let vtt_to_md = get_vtt_to_md_path();
        let (mut cmd, config_root) = new_test_command(&vtt_to_md);
        write_test_config(&config_root, "include_timestamps = \"bogus\"\n");

        let output = cmd
            .arg(input_path.to_str().unwrap())
            .arg("--stdout")
            .output()
            .expect("Failed to execute vtt-to-md");

        assert!(output.status.success());
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(!stdout.contains("[00:00:00.000]"));

        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(stderr.contains("Warning:"));
        assert!(stderr.contains("config.toml"));
    }

    #[test]
    fn legacy_env_value_is_accepted_with_deprecation_warning() {
        let temp_dir = TempDir::new().unwrap();
        let input_path = create_test_vtt(
            &temp_dir,
            "test.vtt",
            "WEBVTT\n\n00:00:00.000 --> 00:00:02.000\n<v Alice>Hello</v>\n",
        );

        let vtt_to_md = get_vtt_to_md_path();
        let (mut cmd, _config_root) = new_test_command(&vtt_to_md);
        let output = cmd
            .arg(input_path.to_str().unwrap())
            .arg("--stdout")
            .env("VTT_TO_MD_INCLUDE_TIMESTAMPS", "first")
            .output()
            .expect("Failed to execute vtt-to-md");

        assert!(output.status.success());
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("[00:00:00.000]"));

        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(stderr.contains("Warning:"));
        assert!(stderr.contains("legacy"));
    }

    #[test]
    fn legacy_config_value_is_accepted_with_deprecation_warning() {
        let temp_dir = TempDir::new().unwrap();
        let input_path = create_test_vtt(
            &temp_dir,
            "test.vtt",
            "WEBVTT\n\n00:00:00.000 --> 00:00:02.000\n<v Alice>Hello</v>\n",
        );

        let vtt_to_md = get_vtt_to_md_path();
        let (mut cmd, config_root) = new_test_command(&vtt_to_md);
        write_test_config(&config_root, "include_timestamps = \"each\"\n");

        let output = cmd
            .arg(input_path.to_str().unwrap())
            .arg("--stdout")
            .output()
            .expect("Failed to execute vtt-to-md");

        assert!(output.status.success());
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("[00:00:00.000]"));

        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(stderr.contains("Warning:"));
        assert!(stderr.contains("legacy"));
    }
}
