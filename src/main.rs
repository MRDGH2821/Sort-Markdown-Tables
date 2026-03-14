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
    cli::{parse_args, InputSource, OutputTarget},
    error::SmtError,
    parser::{parse, Document},
    sorter::{is_table_sorted, sort_document},
    writer::write_document,
};
use std::fs;
use std::io::{self, Read};
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
    let (input_source, output_target, check_mode, _verbose) = match parse_args() {
        Ok(args) => args,
        Err(e) => {
            eprintln!("{}", e);
            return e.exit_code();
        }
    };

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
        // In check mode, determine which files are unsorted
        let mut any_unsorted = false;

        for result in &results {
            if !result.is_sorted {
                any_unsorted = true;
                if let Some(path) = &result.source {
                    eprintln!("{}: file is not sorted", path.display());
                } else {
                    eprintln!("<stdin>: file is not sorted");
                }
            }
        }

        if any_unsorted {
            return 1; // At least one file is unsorted
        } else {
            return 0; // All files are sorted
        }
    }

    // Step 5: Write all documents
    for result in results {
        let target = if let OutputTarget::InPlace = output_target {
            // For in-place, determine the target path from the source file
            match &result.source {
                Some(_path) => OutputTarget::InPlace,
                None => {
                    eprintln!("error: --in-place requires input files (not stdin)");
                    return 2;
                }
            }
        } else {
            output_target.clone()
        };

        // For InPlace, we need to write to the original file path
        let final_target = if let OutputTarget::InPlace = target {
            OutputTarget::File {
                path: result.source.clone().unwrap(),
                append: false,
            }
        } else {
            target
        };

        if let Err(e) = write_document(&result.document, &final_target, result.source.as_deref()) {
            eprintln!("{}", e);
            return 2;
        }
    }

    0 // Success
}

/// Result of processing a file or stdin
#[derive(Debug)]
struct ProcessResult {
    source: Option<PathBuf>,
    document: Document,
    is_sorted: bool,
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

    // Check if already sorted (before modifying)
    let is_sorted = is_document_sorted(&doc);

    // Sort tables
    sort_document(&mut doc)?;

    Ok(ProcessResult {
        source: Some(file_path.clone()),
        document: doc,
        is_sorted,
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

    // Check if already sorted
    let is_sorted = is_document_sorted(&doc);

    // Sort tables
    sort_document(&mut doc)?;

    Ok(ProcessResult {
        source: None,
        document: doc,
        is_sorted,
    })
}

/// Check if a document is already sorted (all SortedTable blocks are sorted).
fn is_document_sorted(doc: &Document) -> bool {
    use smt::parser::Block;

    for block in &doc.blocks {
        if let Block::SortedTable { table, options, .. } = block {
            if !is_table_sorted(table, options) {
                return false;
            }
        }
    }

    true
}

// ============================================================================
// UNIT TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
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
        assert!(!process_result.is_sorted); // Bob before Alice, so unsorted
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
        assert!(process_result.is_sorted); // Already in order
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
        assert!(process_result.is_sorted); // No sorted tables, so "sorted"
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
    fn test_is_document_sorted_all_sorted() {
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
            }],
        };

        assert!(is_document_sorted(&doc));
    }

    #[test]
    fn test_is_document_sorted_one_unsorted() {
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
            }],
        };

        assert!(!is_document_sorted(&doc));
    }

    #[test]
    fn test_is_document_sorted_no_tables() {
        let doc = Document {
            source: None,
            blocks: vec![Block::PlainText(vec!["# Heading".to_string()])],
        };

        assert!(is_document_sorted(&doc));
    }

    #[test]
    fn test_is_document_sorted_mixed_blocks() {
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
                },
                Block::PlainText(vec!["Done.".to_string()]),
            ],
        };

        assert!(is_document_sorted(&doc));
    }

    #[test]
    fn test_process_result_construction() {
        let doc = Document {
            source: None,
            blocks: vec![],
        };

        let result = ProcessResult {
            source: Some(PathBuf::from("test.md")),
            document: doc,
            is_sorted: true,
        };

        assert_eq!(result.source, Some(PathBuf::from("test.md")));
        assert!(result.is_sorted);
    }
}
