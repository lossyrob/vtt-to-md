use std::fs;
use std::path::PathBuf;
use std::process::Command;
use tempfile::TempDir;

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

#[test]
fn test_basic_conversion() {
    let temp_dir = TempDir::new().unwrap();
    let vtt_content = "WEBVTT\n\n00:00:00.000 --> 00:00:02.000\n<v Alice>Hello world</v>\n\n00:00:02.000 --> 00:00:04.000\n<v Bob>Hi there</v>\n";
    let input_path = create_test_vtt(&temp_dir, "test.vtt", vtt_content);
    let output_path = temp_dir.path().join("test.md");

    let vtt_to_md = get_vtt_to_md_path();
    let output = Command::new(&vtt_to_md)
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
    let output = Command::new(&vtt_to_md)
        .arg(input_path.to_str().unwrap())
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
    let output = Command::new(&vtt_to_md)
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
    let output = Command::new(&vtt_to_md)
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
    let output = Command::new(&vtt_to_md)
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
fn test_include_timestamps_first() {
    let temp_dir = TempDir::new().unwrap();
    let vtt_content = "WEBVTT\n\n00:00:00.000 --> 00:00:02.000\n<v Alice>Hello</v>\n\n00:00:02.000 --> 00:00:04.000\n<v Alice>World</v>\n";
    let input_path = create_test_vtt(&temp_dir, "test.vtt", vtt_content);

    let vtt_to_md = get_vtt_to_md_path();
    let output = Command::new(&vtt_to_md)
        .arg(input_path.to_str().unwrap())
        .arg("--include-timestamps")
        .arg("first")
        .arg("--stdout")
        .output()
        .expect("Failed to execute vtt-to-md");

    assert!(
        output.status.success(),
        "Command failed with --include-timestamps first"
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("[00:00:00.000]"));
    assert!(stdout.contains("**Alice:**"));
}

#[test]
fn test_include_timestamps_each() {
    let temp_dir = TempDir::new().unwrap();
    let vtt_content = "WEBVTT\n\n00:00:00.000 --> 00:00:02.000\n<v Alice>Hello</v>\n\n00:00:02.000 --> 00:00:04.000\n<v Alice>World</v>\n";
    let input_path = create_test_vtt(&temp_dir, "test.vtt", vtt_content);

    let vtt_to_md = get_vtt_to_md_path();
    let output = Command::new(&vtt_to_md)
        .arg(input_path.to_str().unwrap())
        .arg("--include-timestamps")
        .arg("each")
        .arg("--stdout")
        .output()
        .expect("Failed to execute vtt-to-md");

    assert!(
        output.status.success(),
        "Command failed with --include-timestamps each"
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("[00:00:00.000]"));
    assert!(stdout.contains("**Alice:**"));
}

#[test]
fn test_file_not_found_error() {
    let vtt_to_md = get_vtt_to_md_path();
    let output = Command::new(&vtt_to_md)
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
    let output = Command::new(&vtt_to_md)
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
    let output = Command::new(&vtt_to_md)
        .arg(input_path.to_str().unwrap())
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
    let output = Command::new(&vtt_to_md)
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
    let output = Command::new(&vtt_to_md)
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
    let output = Command::new(&vtt_to_md)
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
