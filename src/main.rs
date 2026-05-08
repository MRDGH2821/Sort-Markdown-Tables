// SPDX-License-Identifier: GPL-3.0-or-later
// Sort Markdown Tables - A CLI tool for sorting markdown tables
// Copyright (C) 2025 Contributors
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU General Public License for more details.

// ============================================================================
// PHASE 5: Main Module — Pipeline Orchestration
// ============================================================================
//
// Responsibility: Orchestrate the complete pipeline:
// 1. Parse CLI arguments
// 2. Expand glob patterns to file list
// 3. For each file (or stdin):
//    - Read markdown
//    - Parse into Document
//    - Sort tables
//    - Check if modified (for --check mode)
//    - Write output
// 4. Handle errors atomically (no partial writes)
// 5. Return correct exit codes
//
// Key requirements:
// - Exit code 0: Success (or all sorted in check mode)
// - Exit code 1: In check mode, at least one file is unsorted
// - Exit code 2: Error (user input, file I/O, parse error, etc.)
// ============================================================================

use smt::{
    cli::{parse_args, Args, InputSource, OutputTarget},
    error::SmtError,
    parser::{parse, Document},
    sorter::{check_document, sort_document, CheckResult},
    writer::{write_document, write_documents_in_place_atomic},
};
use clap::CommandFactory;
use std::fs;
use std::io::{self, Read};
use std::io::IsTerminal;
use std::path::PathBuf;

// ============================================================================
// TASK 5.4: Main Pipeline Implementation
// ============================================================================

fn main() {
    let exit_code = run();
    std::process::exit(exit_code);
}

/// Main entry point for the application pipeline.
///
/// Algorithm:
/// 1. Parse CLI arguments
/// 2. Based on input source, expand files or use stdin
/// 3. For each file:
///    a. Read contents
///    b. Parse markdown
///    c. Sort tables
///    d. Collect results (document + target)
/// 4. If any error during parsing/sorting: abort, report error, exit 2
/// 5. If --check mode:
///    a. For each result, check if any tables were modified
///    b. Report which files are unsorted
///    c. Exit 1 if any unsorted, 0 if all sorted
/// 6. Otherwise:
///    a. Write all documents to their targets
///    b. Exit 0 on success
fn run() -> i32 {
    // Step 1: Parse CLI arguments
    let (input_source, output_target, check_mode, verbose) = match parse_args() {
        Ok(args) => args,
        Err(e) => {
            eprintln!("{}", e);
            return e.exit_code();
        }
    };

    // Special case: no positional args + TTY stdin -> print help, exit 0.
    // (clap already handles explicit `--help` / `--version`.)
    if matches!(input_source, InputSource::Stdin) && std::io::stdin().is_terminal() {
        let _ = Args::command().print_help();
        println!();
        return 0;
    }

    // Step 2: Prepare list of files to process
    let files_to_process = match &input_source {
        InputSource::Files(files) => files.clone(),
        InputSource::Stdin => {
            // For stdin, we have one "file" with path = None
            vec![]
        }
    };

    // Step 3a: Process all files and collect results
    let mut results: Vec<ProcessResult> = Vec::new();

    // Handle stdin or files
    if files_to_process.is_empty() && matches!(input_source, InputSource::Stdin) {
        // Read from stdin
        match process_stdin(&output_target) {
            Ok(result) => {
                results.push(result);
            }
            Err(e) => {
                eprintln!("{}", e);
                return 2;
            }
        }
    } else {
        // Process file(s)
        for file_path in files_to_process {
            match process_file(&file_path, &output_target) {
                Ok(result) => {
                    results.push(result);
                }
                Err(e) => {
                    eprintln!("{}", e);
                    return 2;
                }
            }
        }
    }

    // Step 4: Check mode branch
    if check_mode {
        let mut unsorted_locations: Vec<CheckResult> = Vec::new();
        for result in &results {
            unsorted_locations.extend(result.check_results.iter().filter(|r| !r.is_sorted).cloned());
        }

        if unsorted_locations.is_empty() {
            return 0;
        }

        // In `--check --verbose`, unsorted table reports go to stdout.
        if verbose {
            for r in unsorted_locations {
                let display_path = match &r.source {
                    Some(p) => p.display().to_string(),
                    None => "<stdin>".to_string(),
                };
                println!(
                    "{}:{}: table is not sorted (comment at line {})",
                    display_path, r.table_start_line, r.comment_line
                );
            }
        }

        return 1;
    }

    // Step 5: Write all documents
    if matches!(output_target, OutputTarget::InPlace) {
        let mut entries = Vec::new();
        for r in results {
            let Some(path) = r.source else {
                eprintln!("{}", SmtError::InPlaceWithStdin);
                return 2;
            };
            entries.push((path, smt::writer::render_document(&r.document)));
        }
        if let Err(e) = write_documents_in_place_atomic(entries) {
            eprintln!("{}", e);
            return 2;
        }
    } else {
        for result in results {
            let target = output_target.clone();

            if let Err(e) = write_document(&result.document, &target, result.source.as_deref()) {
                eprintln!("{}", e);
                return 2;
            }
        }
    }

    0 // Success
}

/// Result of processing a file or stdin
#[derive(Debug)]
struct ProcessResult {
    source: Option<PathBuf>,
    document: Document,
    check_results: Vec<CheckResult>,
}

/// Process a single file: read, parse, sort, and check if modified.
fn process_file(
    file_path: &PathBuf,
    _output_target: &OutputTarget,
) -> Result<ProcessResult, SmtError> {
    // Read file contents
    let contents = fs::read_to_string(file_path).map_err(|e| match e.kind() {
        io::ErrorKind::NotFound => SmtError::FileNotFound {
            path: file_path.clone(),
        },
        io::ErrorKind::PermissionDenied => SmtError::PermissionDenied {
            path: file_path.clone(),
        },
        _ => SmtError::Io { source: e },
    })?;

    // Parse markdown
    let mut doc = parse(&contents, Some(file_path.clone()))?;

    // Check marked tables (before modifying)
    let check_results = check_document(&doc);

    // Sort tables
    sort_document(&mut doc)?;

    Ok(ProcessResult {
        source: Some(file_path.clone()),
        document: doc,
        check_results,
    })
}

/// Process stdin: read from standard input, parse, sort, and write to stdout.
fn process_stdin(output_target: &OutputTarget) -> Result<ProcessResult, SmtError> {
    // Validate that --in-place is not used with stdin
    if matches!(output_target, OutputTarget::InPlace) {
        return Err(SmtError::InPlaceWithStdin);
    }

    // Read from stdin
    let mut contents = String::new();
    io::stdin()
        .read_to_string(&mut contents)
        .map_err(|e| SmtError::Io { source: e })?;

    // Parse markdown (source = None for stdin)
    let mut doc = parse(&contents, None)?;

    // Check marked tables (before modifying)
    let check_results = check_document(&doc);

    // Sort tables
    sort_document(&mut doc)?;

    Ok(ProcessResult {
        source: None,
        document: doc,
        check_results,
    })
}

// ============================================================================
// UNIT TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use smt::parser::LineEnding;
    use smt::parser::{Block, SortOptions, Table, TableRow};

    // ========================================================================
    // TASK 5.4: Main Pipeline Tests
    // ========================================================================

    #[test]
    fn test_process_file_single_table() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let test_file = temp_dir.path().join("test.md");

        let content = "<!-- smt -->\n| Name | Age |\n| --- | --- |\n| Bob | 30 |\n| Alice | 25 |\n";
        fs::write(&test_file, content).unwrap();

        let result = process_file(&test_file, &OutputTarget::Stdout);
        assert!(result.is_ok());

        let process_result = result.unwrap();
        assert_eq!(process_result.source, Some(test_file));
        assert!(process_result.check_results.iter().any(|r| !r.is_sorted)); // Bob before Alice, so unsorted
    }

    #[test]
    fn test_process_file_already_sorted() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let test_file = temp_dir.path().join("test.md");

        let content = "<!-- smt -->\n| Name |\n| --- |\n| Alice |\n| Bob |\n";
        fs::write(&test_file, content).unwrap();

        let result = process_file(&test_file, &OutputTarget::Stdout);
        assert!(result.is_ok());

        let process_result = result.unwrap();
        assert!(process_result.check_results.iter().all(|r| r.is_sorted)); // Already in order
    }

    #[test]
    fn test_process_file_no_tables() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let test_file = temp_dir.path().join("test.md");

        let content = "# Just plain markdown\nNo tables here\n";
        fs::write(&test_file, content).unwrap();

        let result = process_file(&test_file, &OutputTarget::Stdout);
        assert!(result.is_ok());

        let process_result = result.unwrap();
        assert!(process_result.check_results.is_empty()); // No marked tables -> nothing to check
    }

    #[test]
    fn test_process_file_not_found() {
        let missing_file = PathBuf::from("/nonexistent/file.md");

        let result = process_file(&missing_file, &OutputTarget::Stdout);
        assert!(result.is_err());

        match result.unwrap_err() {
            SmtError::FileNotFound { path } => {
                assert_eq!(path, missing_file);
            }
            _ => panic!("Expected FileNotFound error"),
        }
    }

    #[test]
    fn test_check_document_all_sorted() {
        let table = Table {
            start_line: 0,
            header: "| A |".to_string(),
            separator: "| - |".to_string(),
            rows: vec![
                TableRow {
                    raw: "| 1 |".to_string(),
                    cells: vec!["1".to_string()],
                },
                TableRow {
                    raw: "| 2 |".to_string(),
                    cells: vec!["2".to_string()],
                },
            ],
            column_count: 1,
        };

        let doc = Document {
            source: None,
            blocks: vec![Block::SortedTable {
                comment_line: "<!-- smt -->".to_string(),
                comment_line_number: 0,
                options: SortOptions::default(),
                table,
                blank_lines_after_comment: Vec::new(),
            }],
            line_ending: LineEnding::Lf,
            trailing_newline: false,
        };

        let results = check_document(&doc);
        assert_eq!(results.len(), 1);
        assert!(results[0].is_sorted);
    }

    #[test]
    fn test_check_document_one_unsorted() {
        let table = Table {
            start_line: 0,
            header: "| A |".to_string(),
            separator: "| - |".to_string(),
            rows: vec![
                TableRow {
                    raw: "| 2 |".to_string(),
                    cells: vec!["2".to_string()],
                },
                TableRow {
                    raw: "| 1 |".to_string(),
                    cells: vec!["1".to_string()],
                },
            ],
            column_count: 1,
        };

        let doc = Document {
            source: None,
            blocks: vec![Block::SortedTable {
                comment_line: "<!-- smt -->".to_string(),
                comment_line_number: 0,
                options: SortOptions::default(),
                table,
                blank_lines_after_comment: Vec::new(),
            }],
            line_ending: LineEnding::Lf,
            trailing_newline: false,
        };

        let results = check_document(&doc);
        assert_eq!(results.len(), 1);
        assert!(!results[0].is_sorted);
    }

    #[test]
    fn test_check_document_no_tables() {
        let doc = Document {
            source: None,
            blocks: vec![Block::PlainText(vec!["# Heading".to_string()])],
            line_ending: LineEnding::Lf,
            trailing_newline: false,
        };

        let results = check_document(&doc);
        assert!(results.is_empty());
    }

    #[test]
    fn test_check_document_mixed_blocks() {
        let table = Table {
            start_line: 2,
            header: "| A |".to_string(),
            separator: "| - |".to_string(),
            rows: vec![TableRow {
                raw: "| 1 |".to_string(),
                cells: vec!["1".to_string()],
            }],
            column_count: 1,
        };

        let doc = Document {
            source: None,
            blocks: vec![
                Block::PlainText(vec!["# Heading".to_string()]),
                Block::SortedTable {
                    comment_line: "<!-- smt -->".to_string(),
                    comment_line_number: 1,
                    options: SortOptions::default(),
                    table,
                    blank_lines_after_comment: Vec::new(),
                },
                Block::PlainText(vec!["Done.".to_string()]),
            ],
            line_ending: LineEnding::Lf,
            trailing_newline: false,
        };

        let results = check_document(&doc);
        assert_eq!(results.len(), 1);
        assert!(results[0].is_sorted);
    }

    #[test]
    fn test_process_result_construction() {
        let doc = Document {
            source: None,
            blocks: vec![],
            line_ending: LineEnding::Lf,
            trailing_newline: false,
        };

        let result = ProcessResult {
            source: Some(PathBuf::from("test.md")),
            document: doc,
            check_results: Vec::new(),
        };

        assert_eq!(result.source, Some(PathBuf::from("test.md")));
        assert!(result.check_results.is_empty());
    }
}
