// ============================================================================
// Integration Tests for Phase 6: smt CLI Tool
// ============================================================================
//
// Tests invoke the compiled binary with various inputs and verify:
// - Correct sorting by column, type, case sensitivity, order
// - Proper output to stdout, file, or in-place
// - Check mode behavior (exit codes 0, 1)
// - Error handling (exit code 2)
// - Edge cases (empty files, no tables, unmarked tables)
//
// Uses assert_cmd for binary invocation and assertions.
// ============================================================================

use assert_cmd::Command;
use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;

// ============================================================================
// Helper Functions
// ============================================================================

fn fixture_path(dir: &str, name: &str) -> PathBuf {
    PathBuf::from(format!("tests/fixtures/{}/{}", dir, name))
}

fn read_fixture(dir: &str, name: &str) -> String {
    let fixture_file = fixture_path(dir, name);
    let msg = format!("Failed to read fixture {}/{}", dir, name);
    fs::read_to_string(fixture_file).expect(&msg)
}

/// Read fixture and remove trailing newline (for file comparison)
fn read_fixture_without_trailing_newline(dir: &str, name: &str) -> String {
    let mut content = read_fixture(dir, name);
    if content.ends_with('\n') {
        content.pop();
    }
    content
}

// ============================================================================
// Basic Sorting Tests (4 tests)
// ============================================================================

#[test]
fn test_sort_simple_numeric() {
    let input = fixture_path("input", "simple_numeric.md");
    let expected = read_fixture("expected", "simple_numeric.expected.md");

    let mut cmd = Command::cargo_bin("smt").expect("Failed to build binary");
    let output = cmd.arg(&input).output().expect("Failed to execute command");

    assert!(output.status.success(), "Command should exit with 0");

    let stdout = String::from_utf8(output.stdout).expect("Invalid UTF-8");
    assert_eq!(stdout, expected, "Output should match expected");
}

#[test]
fn test_sort_multi_column() {
    let input = fixture_path("input", "multi_column.md");
    let expected = read_fixture("expected", "multi_column.expected.md");

    let mut cmd = Command::cargo_bin("smt").expect("Failed to build binary");
    let output = cmd.arg(&input).output().expect("Failed to execute command");

    assert!(output.status.success(), "Command should exit with 0");

    let stdout = String::from_utf8(output.stdout).expect("Invalid UTF-8");
    assert_eq!(stdout, expected, "Output should match expected");
}

#[test]
fn test_sort_case_insensitive() {
    let input = fixture_path("input", "case_insensitive.md");
    let expected = read_fixture("expected", "case_insensitive.expected.md");

    let mut cmd = Command::cargo_bin("smt").expect("Failed to build binary");
    let output = cmd.arg(&input).output().expect("Failed to execute command");

    assert!(output.status.success(), "Command should exit with 0");

    let stdout = String::from_utf8(output.stdout).expect("Invalid UTF-8");
    assert_eq!(stdout, expected, "Output should match expected");
}

#[test]
fn test_sort_descending() {
    let input = fixture_path("input", "descending.md");
    let expected = read_fixture("expected", "descending.expected.md");

    let mut cmd = Command::cargo_bin("smt").expect("Failed to build binary");
    let output = cmd.arg(&input).output().expect("Failed to execute command");

    assert!(output.status.success(), "Command should exit with 0");

    let stdout = String::from_utf8(output.stdout).expect("Invalid UTF-8");
    assert_eq!(stdout, expected, "Output should match expected");
}

// ============================================================================
// Output Target Tests (3 tests)
// ============================================================================

#[test]
fn test_output_to_stdout() {
    let input = fixture_path("input", "simple_numeric.md");
    let expected = read_fixture("expected", "simple_numeric.expected.md");

    let mut cmd = Command::cargo_bin("smt").expect("Failed to build binary");
    cmd.arg(&input);

    cmd.assert().success();
    cmd.assert().stdout(expected);
}

#[test]
fn test_output_to_file() {
    let tmp_dir = TempDir::new().expect("Failed to create temp dir");
    let output_path = tmp_dir.path().join("output.md");

    let input = fixture_path("input", "simple_numeric.md");
    let expected = read_fixture_without_trailing_newline("expected", "simple_numeric.expected.md");

    let mut cmd = Command::cargo_bin("smt").expect("Failed to build binary");
    cmd.arg(&input).arg("-w").arg(&output_path);

    cmd.assert().success();

    let output_content = fs::read_to_string(&output_path).expect("Failed to read output file");
    assert_eq!(
        output_content, expected,
        "Output file should match expected"
    );
}

#[test]
fn test_inplace_sort() {
    let tmp_dir = TempDir::new().expect("Failed to create temp dir");
    let work_file = tmp_dir.path().join("work.md");

    let input = fixture_path("input", "simple_numeric.md");
    let expected = read_fixture_without_trailing_newline("expected", "simple_numeric.expected.md");

    // Copy fixture to temp location
    fs::copy(&input, &work_file).expect("Failed to copy fixture");

    let mut cmd = Command::cargo_bin("smt").expect("Failed to build binary");
    cmd.arg("-i").arg(&work_file);

    cmd.assert().success();

    let result = fs::read_to_string(&work_file).expect("Failed to read result");
    assert_eq!(result, expected, "File should be sorted in-place");
}

// ============================================================================
// Check Mode Tests (3 tests)
// ============================================================================

#[test]
fn test_check_sorted_file_exits_0() {
    let input = fixture_path("expected", "simple_numeric.expected.md");

    let mut cmd = Command::cargo_bin("smt").expect("Failed to build binary");
    cmd.arg(&input).arg("--check");

    cmd.assert().success();
}

#[test]
fn test_check_unsorted_file_exits_1() {
    let input = fixture_path("unsorted", "unsorted_numeric.md");

    let mut cmd = Command::cargo_bin("smt").expect("Failed to build binary");
    cmd.arg(&input).arg("--check");

    cmd.assert().failure().code(1);
}

#[test]
fn test_check_mixed_sorted_unsorted_exits_1() {
    // Create a temp dir with mixed files
    let tmp_dir = TempDir::new().expect("Failed to create temp dir");
    let sorted_file = tmp_dir.path().join("sorted.md");
    let unsorted_file = tmp_dir.path().join("unsorted.md");

    let sorted_input = fixture_path("expected", "simple_numeric.expected.md");
    let unsorted_input = fixture_path("unsorted", "unsorted_numeric.md");

    fs::copy(&sorted_input, &sorted_file).expect("Failed to copy sorted fixture");
    fs::copy(&unsorted_input, &unsorted_file).expect("Failed to copy unsorted fixture");

    let mut cmd = Command::cargo_bin("smt").expect("Failed to build binary");
    cmd.arg(&sorted_file).arg(&unsorted_file).arg("--check");

    cmd.assert().failure().code(1);
}

// ============================================================================
// Multiple Tables Test (1 test)
// ============================================================================

#[test]
fn test_multiple_tables_in_one_file() {
    let input = fixture_path("input", "multiple_tables.md");
    let expected = read_fixture("expected", "multiple_tables.expected.md");

    let mut cmd = Command::cargo_bin("smt").expect("Failed to build binary");
    let output = cmd.arg(&input).output().expect("Failed to execute command");

    assert!(output.status.success(), "Command should exit with 0");

    let stdout = String::from_utf8(output.stdout).expect("Invalid UTF-8");
    assert_eq!(stdout, expected, "Both tables should be sorted");
}

// ============================================================================
// Edge Case Tests (4 tests)
// ============================================================================

#[test]
fn test_empty_file_unchanged() {
    let tmp_dir = TempDir::new().expect("Failed to create temp dir");
    let empty_file = tmp_dir.path().join("empty.md");

    // Create an empty file
    fs::write(&empty_file, "").expect("Failed to write empty file");

    let mut cmd = Command::cargo_bin("smt").expect("Failed to build binary");
    let output = cmd
        .arg(&empty_file)
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success(), "Empty file should succeed");

    let stdout = String::from_utf8(output.stdout).expect("Invalid UTF-8");
    // Empty file with println! outputs just a newline
    assert_eq!(stdout, "\n", "Empty file should output single newline");
}

#[test]
fn test_no_tables_unchanged() {
    let input = fixture_path("input", "no_tables.md");
    let expected = read_fixture("expected", "no_tables.expected.md");

    let mut cmd = Command::cargo_bin("smt").expect("Failed to build binary");
    let output = cmd.arg(&input).output().expect("Failed to execute command");

    assert!(
        output.status.success(),
        "File with no tables should succeed"
    );

    let stdout = String::from_utf8(output.stdout).expect("Invalid UTF-8");
    assert_eq!(stdout, expected, "File should be unchanged");
}

#[test]
fn test_unmarked_table_unchanged() {
    let input = fixture_path("input", "unmarked_table.md");
    let expected = read_fixture("expected", "unmarked_table.expected.md");

    let mut cmd = Command::cargo_bin("smt").expect("Failed to build binary");
    let output = cmd.arg(&input).output().expect("Failed to execute command");

    assert!(
        output.status.success(),
        "File with unmarked table should succeed"
    );

    let stdout = String::from_utf8(output.stdout).expect("Invalid UTF-8");
    assert_eq!(stdout, expected, "Unmarked table should not be sorted");
}

#[test]
fn test_mixed_numeric_values() {
    let input = fixture_path("input", "mixed_numeric.md");
    let expected = read_fixture("expected", "mixed_numeric.expected.md");

    let mut cmd = Command::cargo_bin("smt").expect("Failed to build binary");
    let output = cmd.arg(&input).output().expect("Failed to execute command");

    assert!(
        output.status.success(),
        "File with mixed values should succeed"
    );

    let stdout = String::from_utf8(output.stdout).expect("Invalid UTF-8");
    assert_eq!(stdout, expected, "Numeric values should come first");
}

// ============================================================================
// Error Handling Tests (3 tests)
// ============================================================================

#[test]
fn test_nonexistent_file_exits_2() {
    let mut cmd = Command::cargo_bin("smt").expect("Failed to build binary");
    cmd.arg("/nonexistent/path/file.md");

    cmd.assert().failure().code(2);
}

#[test]
fn test_invalid_args_exits_2() {
    let mut cmd = Command::cargo_bin("smt").expect("Failed to build binary");
    // Conflicting arguments
    cmd.arg("-i")
        .arg("-w")
        .arg("output.md")
        .arg("tests/fixtures/input/simple_numeric.md");

    cmd.assert().failure().code(2);
}

#[test]
fn test_check_with_output_arg_conflicts() {
    let input = fixture_path("input", "simple_numeric.md");

    let mut cmd = Command::cargo_bin("smt").expect("Failed to build binary");
    cmd.arg(&input).arg("--check").arg("-w").arg("output.md");

    cmd.assert().failure().code(2);
}

// ============================================================================
// File I/O Tests (2 tests)
// ============================================================================

#[test]
fn test_stdin_to_stdout() {
    let input_content = read_fixture("input", "simple_numeric.md");
    let expected = read_fixture("expected", "simple_numeric.expected.md");

    let mut cmd = Command::cargo_bin("smt").expect("Failed to build binary");
    let output = cmd
        .write_stdin(input_content)
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success(), "Stdin should succeed");

    let stdout = String::from_utf8(output.stdout).expect("Invalid UTF-8");
    assert_eq!(stdout, expected, "Stdin output should match expected");
}

#[test]
fn test_inplace_with_stdin_error() {
    let input_content = read_fixture("input", "simple_numeric.md");

    let mut cmd = Command::cargo_bin("smt").expect("Failed to build binary");
    cmd.arg("-i").write_stdin(input_content);

    cmd.assert().failure().code(2);
}
