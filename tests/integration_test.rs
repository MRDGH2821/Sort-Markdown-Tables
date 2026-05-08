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
#[cfg(unix)]
use {std::fs::Permissions, std::os::unix::fs::PermissionsExt};

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

#[test]
fn test_inplace_sort_multiple_files() {
    let tmp_dir = TempDir::new().expect("Failed to create temp dir");
    let work_a = tmp_dir.path().join("a.md");
    let work_b = tmp_dir.path().join("b.md");

    let input = fixture_path("input", "simple_numeric.md");
    let expected = read_fixture_without_trailing_newline("expected", "simple_numeric.expected.md");

    fs::copy(&input, &work_a).expect("Failed to copy fixture a");
    fs::copy(&input, &work_b).expect("Failed to copy fixture b");

    let mut cmd = Command::cargo_bin("smt").expect("Failed to build binary");
    cmd.arg("-i").arg(&work_a).arg(&work_b);

    cmd.assert().success();

    let a = fs::read_to_string(&work_a).expect("Failed to read a");
    let b = fs::read_to_string(&work_b).expect("Failed to read b");
    assert_eq!(a, expected, "a should be sorted in-place");
    assert_eq!(b, expected, "b should be sorted in-place");
}

#[test]
fn test_inplace_atomicity_parse_error_prevents_any_write() {
    let tmp_dir = TempDir::new().expect("Failed to create temp dir");
    let ok_file = tmp_dir.path().join("ok.md");
    let bad_file = tmp_dir.path().join("bad.md");

    let ok_input = fixture_path("input", "simple_numeric.md");
    fs::copy(&ok_input, &ok_file).expect("Failed to copy ok fixture");
    fs::write(&bad_file, "<!-- smt -->\nthis is not a table\n").expect("Failed to write bad file");

    let ok_before = fs::read_to_string(&ok_file).expect("Failed to read ok before");
    let bad_before = fs::read_to_string(&bad_file).expect("Failed to read bad before");

    let mut cmd = Command::cargo_bin("smt").expect("Failed to build binary");
    cmd.arg("-i").arg(&ok_file).arg(&bad_file);
    cmd.assert().failure().code(2);

    let ok_after = fs::read_to_string(&ok_file).expect("Failed to read ok after");
    let bad_after = fs::read_to_string(&bad_file).expect("Failed to read bad after");

    assert_eq!(
        ok_after, ok_before,
        "ok file must remain unchanged on error"
    );
    assert_eq!(
        bad_after, bad_before,
        "bad file must remain unchanged on error"
    );
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

#[test]
fn test_check_verbose_prints_unsorted_locations() {
    let input = fixture_path("unsorted", "unsorted_numeric.md");

    let mut cmd = Command::cargo_bin("smt").expect("Failed to build binary");
    cmd.arg(&input).arg("--check").arg("--verbose");

    cmd.assert()
        .failure()
        .code(1)
        .stdout(predicates::str::contains("table is not sorted"));
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

#[test]
fn test_check_with_in_place_errors_exit_2() {
    let input = fixture_path("input", "simple_numeric.md");

    let mut cmd = Command::cargo_bin("smt").expect("Failed to build binary");
    cmd.arg("--check").arg("--in-place").arg(&input);

    cmd.assert().failure().code(2);
}

#[test]
fn test_write_with_multiple_input_files_errors_exit_2() {
    let tmp_dir = TempDir::new().expect("Failed to create temp dir");
    let out = tmp_dir.path().join("out.md");
    let a = fixture_path("input", "simple_numeric.md");
    let b = fixture_path("input", "no_tables.md");

    let mut cmd = Command::cargo_bin("smt").expect("Failed to build binary");
    cmd.arg(&a)
        .arg(&b)
        .arg("-w")
        .arg(&out)
        .assert()
        .failure()
        .code(2)
        .stderr(predicates::str::contains(
            "--write cannot be used with multiple input files",
        ));
}

#[test]
fn test_append_without_write_errors() {
    let mut cmd = Command::cargo_bin("smt").expect("Failed to build binary");
    cmd.arg("--append")
        .assert()
        .failure()
        .stderr(predicates::str::contains("--append"));
}

#[test]
fn test_append_writes_after_existing_file_content() {
    let tmp_dir = TempDir::new().expect("Failed to create temp dir");
    let out = tmp_dir.path().join("out.md");
    fs::write(&out, "PREFIX_LINE\n").expect("Failed to write prefix");

    let input = fixture_path("unsorted", "unsorted_numeric.md");
    let golden = {
        let output = Command::cargo_bin("smt")
            .expect("Failed to build binary")
            .arg(&input)
            .output()
            .expect("Failed to capture stdout baseline");
        assert!(
            output.status.success(),
            "baseline sort failed:\nstderr={}",
            String::from_utf8_lossy(&output.stderr)
        );
        let mut g = String::from_utf8(output.stdout).expect("Invalid UTF-8");
        while g.ends_with('\n') {
            g.pop();
        }
        g
    };

    let mut cmd = Command::cargo_bin("smt").expect("Failed to build binary");
    cmd.arg(&input)
        .arg("--append")
        .arg("-w")
        .arg(&out)
        .assert()
        .success();

    let combined = fs::read_to_string(&out).expect("Failed to read output file");
    assert_eq!(
        combined,
        format!("PREFIX_LINE\n{}", golden),
        "append mode should preserve prior bytes then write rendered document"
    );
}

#[test]
fn test_stdin_with_write_targets_file() {
    let tmp_dir = TempDir::new().expect("Failed to create temp dir");
    let out = tmp_dir.path().join("out.md");
    let input_content = read_fixture("input", "simple_numeric.md");
    let expected = read_fixture_without_trailing_newline("expected", "simple_numeric.expected.md");

    let mut cmd = Command::cargo_bin("smt").expect("Failed to build binary");
    cmd.write_stdin(input_content)
        .arg("-w")
        .arg(&out)
        .assert()
        .success();

    let got = fs::read_to_string(&out).expect("Failed to read output");
    assert_eq!(got, expected);
}

#[test]
fn test_stable_sort_preserves_order_of_equal_keys() {
    let tmp_dir = TempDir::new().expect("Failed to create temp dir");
    let file = tmp_dir.path().join("stable.md");
    let body = "<!-- smt column=1 type=numeric -->\n\
        | k | tag |\n\
        | - | --- |\n\
        | 2 | z   |\n\
        | 1 | a   |\n\
        | 1 | b   |\n";
    fs::write(&file, body).expect("Failed to write stable fixture");

    let mut cmd = Command::cargo_bin("smt").expect("Failed to build binary");
    let output = cmd.arg(&file).output().expect("Failed to run smt");
    assert!(
        output.status.success(),
        "stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).expect("Invalid UTF-8");
    let pos_a = stdout
        .find("| 1 | a   |")
        .expect("expected first equal key row");
    let pos_b = stdout
        .find("| 1 | b   |")
        .expect("expected second equal key row");
    assert!(
        pos_a < pos_b,
        "stable sort must preserve relative order among equal keys (stdout={stdout:?})"
    );
}

#[test]
fn test_invalid_smt_comment_option_exit_2() {
    let tmp_dir = TempDir::new().expect("Failed to create temp dir");
    let file = tmp_dir.path().join("badopt.md");
    fs::write(
        &file,
        "<!-- smt not_an_option=value -->\n| A |\n| - |\n| x |\n",
    )
    .expect("write");

    let mut cmd = Command::cargo_bin("smt").expect("Failed to build binary");
    cmd.arg(&file)
        .assert()
        .failure()
        .code(2)
        .stderr(predicates::str::contains("unknown option"));
}

#[test]
fn test_column_out_of_range_in_comment_exit_2() {
    let tmp_dir = TempDir::new().expect("Failed to create temp dir");
    let file = tmp_dir.path().join("badcol.md");
    fs::write(&file, "<!-- smt column=99 -->\n| A |\n| - |\n| x |\n").expect("write");

    let mut cmd = Command::cargo_bin("smt").expect("Failed to build binary");
    cmd.arg(&file)
        .assert()
        .failure()
        .code(2)
        .stderr(predicates::str::contains("out of range"));
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
fn test_preserves_crlf_line_endings() {
    let tmp_dir = TempDir::new().expect("Failed to create temp dir");
    let file = tmp_dir.path().join("crlf.md");

    // Minimal table with CRLF line endings.
    let crlf = "<!-- smt -->\r\n| A |\r\n| - |\r\n| 2 |\r\n| 1 |\r\n";
    fs::write(&file, crlf).expect("Failed to write crlf fixture");

    let mut cmd = Command::cargo_bin("smt").expect("Failed to build binary");
    let output = cmd.arg(&file).output().expect("Failed to execute command");
    assert!(
        output.status.success(),
        "Command failed.\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    // Verify the output still contains CRLF.
    let stdout = String::from_utf8(output.stdout).expect("Invalid UTF-8");
    assert!(
        stdout.contains("\r\n"),
        "Expected CRLF line endings in stdout"
    );
}

#[test]
fn test_unicode_characters_sorted_correctly() {
    let tmp_dir = TempDir::new().expect("Failed to create temp dir");
    let file = tmp_dir.path().join("unicode.md");

    // Use lexicographic + case-insensitive to exercise Unicode lowercasing.
    let values = vec!["Zebra", "ångström", "Äpfel", "ábaco"];
    let mut expected_values = values.clone();
    expected_values.sort_by(|a, b| a.to_lowercase().cmp(&b.to_lowercase()));

    let input = format!(
        "<!-- smt type=lexicographic case=insensitive -->\n| Word |\n| ---- |\n{}\n",
        values
            .iter()
            .map(|v| format!("| {} |", v))
            .collect::<Vec<_>>()
            .join("\n")
    );
    fs::write(&file, input).expect("Failed to write unicode fixture");

    let mut expected =
        String::from("<!-- smt type=lexicographic case=insensitive -->\n| Word |\n| ---- |\n");
    for v in expected_values {
        expected.push_str(&format!("| {} |\n", v));
    }

    let mut cmd = Command::cargo_bin("smt").expect("Failed to build binary");
    let output = cmd.arg(&file).output().expect("Failed to execute command");
    assert!(
        output.status.success(),
        "Command failed.\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8(output.stdout).expect("Invalid UTF-8");
    assert_eq!(stdout, expected, "Unicode lexicographic sort should match");
}

#[cfg(unix)]
#[test]
fn test_permission_denied_on_file_read_exits_2() {
    let tmp_dir = TempDir::new().expect("Failed to create temp dir");
    let file = tmp_dir.path().join("no_read.md");

    fs::write(&file, "# hi\n").expect("Failed to write fixture");
    fs::set_permissions(&file, Permissions::from_mode(0o000))
        .expect("Failed to chmod fixture to 000");

    let mut cmd = Command::cargo_bin("smt").expect("Failed to build binary");
    cmd.arg(&file).assert().failure().code(2);

    // Restore so tempdir cleanup doesn't fail.
    fs::set_permissions(&file, Permissions::from_mode(0o600))
        .expect("Failed to restore permissions");
}

#[cfg(unix)]
#[test]
fn test_permission_denied_on_file_write_exits_2() {
    let tmp_dir = TempDir::new().expect("Failed to create temp dir");
    let input = fixture_path("input", "simple_numeric.md");

    let out_dir = tmp_dir.path().join("no_write_dir");
    fs::create_dir_all(&out_dir).expect("Failed to create output dir");
    fs::set_permissions(&out_dir, Permissions::from_mode(0o555))
        .expect("Failed to chmod output dir to 555");

    let output_path = out_dir.join("out.md");

    let mut cmd = Command::cargo_bin("smt").expect("Failed to build binary");
    cmd.arg(&input)
        .arg("-w")
        .arg(&output_path)
        .assert()
        .failure()
        .code(2);

    // Restore so tempdir cleanup doesn't fail.
    fs::set_permissions(&out_dir, Permissions::from_mode(0o755))
        .expect("Failed to restore dir permissions");
}

#[test]
fn test_glob_pattern_matching_multiple_files_processed() {
    let tmp_dir = TempDir::new().expect("Failed to create temp dir");

    let a = tmp_dir.path().join("a.md");
    let b = tmp_dir.path().join("b.md");
    let expected_sorted = read_fixture("expected", "simple_numeric.expected.md");

    fs::write(&a, &expected_sorted).expect("Failed to write a");
    fs::write(&b, &expected_sorted).expect("Failed to write b");

    let pattern = format!("{}/*.md", tmp_dir.path().display());

    let mut cmd = Command::cargo_bin("smt").expect("Failed to build binary");
    cmd.arg("--check").arg(pattern).assert().success().code(0);
}

#[test]
fn test_glob_with_zero_matches_errors_exit_2() {
    let tmp_dir = TempDir::new().expect("Failed to create temp dir");
    let pattern = format!("{}/nope-*.md", tmp_dir.path().display());

    let mut cmd = Command::cargo_bin("smt").expect("Failed to build binary");
    cmd.arg("--check")
        .arg(&pattern)
        .assert()
        .failure()
        .code(2)
        .stderr(predicates::str::contains("no files matched pattern"));
}

#[test]
fn test_inplace_with_stdin_error() {
    let input_content = read_fixture("input", "simple_numeric.md");

    let mut cmd = Command::cargo_bin("smt").expect("Failed to build binary");
    cmd.arg("-i").write_stdin(input_content);

    cmd.assert().failure().code(2);
}
