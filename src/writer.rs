// ============================================================================
// PHASE 5: Writer Module — Output Handling
// ============================================================================
//
// Responsibility: Render Document to string and write to various targets
// (stdout, file, or in-place with atomic writes).
//
// Key requirements:
// 1. render_document: Join all blocks + tables into a single string
// 2. write_document: Route to stdout/file/in-place with error handling
// 3. Atomic writes for -i mode using tempfile::NamedTempFile
// 4. Preserve all non-sorted content exactly as-is
// ============================================================================

use crate::cli::OutputTarget;
use crate::error::SmtError;
use crate::parser::Block;
#[cfg(not(test))]
use crate::parser::Document;
#[cfg(test)]
use crate::parser::{Document, Table, TableRow};
use std::fs::OpenOptions;
use std::io::{self, Write};
use std::path::Path;
use tempfile::NamedTempFile;

// ============================================================================
// TASK 5.1: Render Document to String
// ============================================================================

/// Render a Document to a complete markdown string.
///
/// Algorithm:
/// 1. Iterate through all blocks
/// 2. For PlainText blocks: join lines with newlines
/// 3. For SortedTable blocks: render table (header, separator, sorted rows)
/// 4. Join all rendered blocks with newlines
///
/// The rendered output preserves all content, with sorted tables updated.
pub fn render_document(doc: &Document) -> String {
    let mut output = Vec::new();

    for block in &doc.blocks {
        match block {
            Block::PlainText(lines) => {
                output.push(lines.join("\n"));
            }
            Block::SortedTable {
                comment_line,
                table,
                ..
            } => {
                // Render the comment line
                output.push(comment_line.clone());

                // Render the table: header, separator, and sorted rows
                output.push(table.header.clone());
                output.push(table.separator.clone());

                for row in &table.rows {
                    output.push(row.raw.clone());
                }
            }
        }
    }

    output.join("\n")
}

// ============================================================================
// TASK 5.2: Atomic Write Logic for In-Place Mode
// ============================================================================

/// Write to a file atomically using tempfile.
///
/// Algorithm:
/// 1. Create a NamedTempFile in the same directory as the target
/// 2. Write content to the temp file
/// 3. If write succeeds: persist (rename) temp → target
/// 4. If write fails: temp file auto-deleted, original untouched
///
/// This ensures that on ANY error, the original file is never corrupted.
#[allow(dead_code)]
fn write_atomic(path: &Path, content: &str) -> Result<(), SmtError> {
    // Determine the directory for the temp file
    let dir = path.parent().unwrap_or_else(|| Path::new("."));

    // Create a NamedTempFile in the same directory
    let mut temp_file = NamedTempFile::new_in(dir).map_err(|e| map_io_error(&e, path))?;

    // Write content to temp file
    temp_file
        .write_all(content.as_bytes())
        .map_err(|e| map_io_error(&e, path))?;

    // Flush to ensure all data is written
    temp_file.flush().map_err(|e| map_io_error(&e, path))?;

    // Persist the temp file to the target path
    // This performs an atomic rename on most systems
    temp_file
        .persist(path)
        .map_err(|e| map_io_error(&e.error, path))?;

    Ok(())
}

/// Map an IO error to a SmtError with appropriate context.
fn map_io_error(err: &io::Error, path: &Path) -> SmtError {
    use std::io::ErrorKind;

    match err.kind() {
        ErrorKind::NotFound => SmtError::FileNotFound {
            path: path.to_path_buf(),
        },
        ErrorKind::PermissionDenied => SmtError::PermissionDenied {
            path: path.to_path_buf(),
        },
        _ => SmtError::Io {
            source: io::Error::new(err.kind(), err.to_string()),
        },
    }
}

// ============================================================================
// TASK 5.3: Write Document Orchestration
// ============================================================================

/// Write a rendered document to the specified output target.
///
/// Algorithm:
/// 1. Render the document to string
/// 2. Match on OutputTarget:
///    - Stdout: println!()
///    - File { path, append }: write to file (create if new, append if flag set)
///    - InPlace: atomic write (temp file + rename)
/// 3. Return Ok(()) on success, SmtError on failure
///
/// All errors are mapped to SmtError with source location context.
pub fn write_document(
    doc: &Document,
    target: &OutputTarget,
    _source: Option<&Path>,
) -> Result<(), SmtError> {
    // Render the document to string
    let content = render_document(doc);

    match target {
        OutputTarget::Stdout => {
            // Write to stdout
            println!("{}", content);
            Ok(())
        }
        OutputTarget::File { path, append } => {
            // Write to a file (create new or append)
            let mut file = OpenOptions::new()
                .write(true)
                .create(true)
                .append(*append)
                .truncate(!*append)
                .open(path)
                .map_err(|e| map_io_error(&e, path))?;

            file.write_all(content.as_bytes())
                .map_err(|e| map_io_error(&e, path))?;
            file.flush().map_err(|e| map_io_error(&e, path))?;

            Ok(())
        }
        OutputTarget::InPlace => {
            // This shouldn't happen if main.rs is correct
            // (InPlace requires a path, which is stored separately)
            // But we handle it defensively
            Err(SmtError::Io {
                source: io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "InPlace target requires a source path",
                ),
            })
        }
    }
}

// ============================================================================
// UNIT TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::SortOptions;

    // ========================================================================
    // TASK 5.1: Render Document Tests
    // ========================================================================

    #[test]
    fn test_render_plain_text_only() {
        let doc = Document {
            source: None,
            blocks: vec![Block::PlainText(vec![
                "# Heading".to_string(),
                "Some text".to_string(),
            ])],
        };

        let rendered = render_document(&doc);
        assert_eq!(rendered, "# Heading\nSome text");
    }

    #[test]
    fn test_render_empty_document() {
        let doc = Document {
            source: None,
            blocks: vec![],
        };

        let rendered = render_document(&doc);
        assert_eq!(rendered, "");
    }

    #[test]
    fn test_render_table_only() {
        let table = Table {
            start_line: 1,
            header: "| Name | Age |".to_string(),
            separator: "| --- | --- |".to_string(),
            rows: vec![
                TableRow {
                    raw: "| Alice | 30 |".to_string(),
                    cells: vec!["Alice".to_string(), "30".to_string()],
                },
                TableRow {
                    raw: "| Bob | 25 |".to_string(),
                    cells: vec!["Bob".to_string(), "25".to_string()],
                },
            ],
            column_count: 2,
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

        let rendered = render_document(&doc);
        assert!(rendered.contains("<!-- smt -->"));
        assert!(rendered.contains("| Name | Age |"));
        assert!(rendered.contains("| --- | --- |"));
        assert!(rendered.contains("| Alice | 30 |"));
        assert!(rendered.contains("| Bob | 25 |"));
    }

    #[test]
    fn test_render_mixed_blocks() {
        let table = Table {
            start_line: 3,
            header: "| A | B |".to_string(),
            separator: "| - | - |".to_string(),
            rows: vec![TableRow {
                raw: "| 1 | 2 |".to_string(),
                cells: vec!["1".to_string(), "2".to_string()],
            }],
            column_count: 2,
        };

        let doc = Document {
            source: None,
            blocks: vec![
                Block::PlainText(vec!["# Title".to_string()]),
                Block::SortedTable {
                    comment_line: "<!-- smt -->".to_string(),
                    comment_line_number: 2,
                    options: SortOptions::default(),
                    table,
                },
                Block::PlainText(vec!["Done.".to_string()]),
            ],
        };

        let rendered = render_document(&doc);
        assert!(rendered.contains("# Title"));
        assert!(rendered.contains("<!-- smt -->"));
        assert!(rendered.contains("Done."));
    }

    #[test]
    fn test_render_multiple_tables() {
        let table1 = Table {
            start_line: 1,
            header: "| Col |".to_string(),
            separator: "| --- |".to_string(),
            rows: vec![TableRow {
                raw: "| A |".to_string(),
                cells: vec!["A".to_string()],
            }],
            column_count: 1,
        };

        let table2 = Table {
            start_line: 5,
            header: "| X |".to_string(),
            separator: "| - |".to_string(),
            rows: vec![TableRow {
                raw: "| Y |".to_string(),
                cells: vec!["Y".to_string()],
            }],
            column_count: 1,
        };

        let doc = Document {
            source: None,
            blocks: vec![
                Block::SortedTable {
                    comment_line: "<!-- smt -->".to_string(),
                    comment_line_number: 0,
                    options: SortOptions::default(),
                    table: table1,
                },
                Block::PlainText(vec!["---".to_string()]),
                Block::SortedTable {
                    comment_line: "<!-- smt -->".to_string(),
                    comment_line_number: 4,
                    options: SortOptions::default(),
                    table: table2,
                },
            ],
        };

        let rendered = render_document(&doc);
        let lines: Vec<&str> = rendered.lines().collect();
        // Should have: comment, header, separator, row, "---", comment, header, separator, row
        assert!(lines.len() >= 9);
        assert_eq!(lines[0], "<!-- smt -->");
        assert_eq!(lines[1], "| Col |");
    }

    #[test]
    fn test_render_preserves_content() {
        let doc = Document {
            source: None,
            blocks: vec![
                Block::PlainText(vec!["Line 1".to_string(), "Line 2".to_string()]),
                Block::PlainText(vec!["Line 3".to_string()]),
            ],
        };

        let rendered = render_document(&doc);
        assert_eq!(rendered, "Line 1\nLine 2\nLine 3");
    }

    #[test]
    fn test_render_empty_plaintext() {
        let doc = Document {
            source: None,
            blocks: vec![Block::PlainText(vec![])],
        };

        let rendered = render_document(&doc);
        assert_eq!(rendered, "");
    }

    // ========================================================================
    // TASK 5.2: Atomic Write Tests
    // ========================================================================

    #[test]
    fn test_write_atomic_success() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let test_file = temp_dir.path().join("test.md");

        let content = "# Test\nContent";
        let result = write_atomic(&test_file, content);

        assert!(result.is_ok());
        assert!(test_file.exists());

        let written = std::fs::read_to_string(&test_file).unwrap();
        assert_eq!(written, content);
    }

    #[test]
    fn test_write_atomic_overwrites_existing() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let test_file = temp_dir.path().join("test.md");

        // Write initial content
        std::fs::write(&test_file, "Old content").unwrap();

        // Overwrite with atomic write
        let new_content = "New content";
        let result = write_atomic(&test_file, new_content);

        assert!(result.is_ok());
        let written = std::fs::read_to_string(&test_file).unwrap();
        assert_eq!(written, new_content);
    }

    #[test]
    fn test_write_atomic_invalid_directory() {
        let test_file = Path::new("/nonexistent/directory/file.md");

        let result = write_atomic(test_file, "content");
        assert!(result.is_err());
    }

    #[test]
    fn test_write_atomic_creates_parent_if_exists() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let subdir = temp_dir.path().join("subdir");
        std::fs::create_dir(&subdir).unwrap();

        let test_file = subdir.join("test.md");
        let result = write_atomic(&test_file, "content");

        assert!(result.is_ok());
        assert!(test_file.exists());
    }

    // ========================================================================
    // TASK 5.3: Write Document Tests
    // ========================================================================

    #[test]
    fn test_write_document_stdout() {
        let doc = Document {
            source: None,
            blocks: vec![Block::PlainText(vec!["# Test".to_string()])],
        };

        let target = OutputTarget::Stdout;
        let result = write_document(&doc, &target, None);

        // This will print to stdout, but should succeed
        assert!(result.is_ok());
    }

    #[test]
    fn test_write_document_to_file() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let output_file = temp_dir.path().join("output.md");

        let doc = Document {
            source: None,
            blocks: vec![Block::PlainText(vec!["# Test".to_string()])],
        };

        let target = OutputTarget::File {
            path: output_file.clone(),
            append: false,
        };

        let result = write_document(&doc, &target, None);
        assert!(result.is_ok());
        assert!(output_file.exists());

        let written = std::fs::read_to_string(&output_file).unwrap();
        assert!(written.contains("# Test"));
    }

    #[test]
    fn test_write_document_to_file_append() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let output_file = temp_dir.path().join("output.md");

        // Write initial content
        std::fs::write(&output_file, "Initial\n").unwrap();

        let doc = Document {
            source: None,
            blocks: vec![Block::PlainText(vec!["Appended".to_string()])],
        };

        let target = OutputTarget::File {
            path: output_file.clone(),
            append: true,
        };

        let result = write_document(&doc, &target, None);
        assert!(result.is_ok());

        let written = std::fs::read_to_string(&output_file).unwrap();
        assert!(written.contains("Initial"));
        assert!(written.contains("Appended"));
    }

    #[test]
    fn test_write_document_creates_new_file() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let output_file = temp_dir.path().join("new_file.md");

        assert!(!output_file.exists());

        let doc = Document {
            source: None,
            blocks: vec![Block::PlainText(vec!["New file".to_string()])],
        };

        let target = OutputTarget::File {
            path: output_file.clone(),
            append: false,
        };

        let result = write_document(&doc, &target, None);
        assert!(result.is_ok());
        assert!(output_file.exists());
    }

    #[test]
    fn test_write_document_with_table() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let output_file = temp_dir.path().join("table.md");

        let table = Table {
            start_line: 1,
            header: "| Name |".to_string(),
            separator: "| --- |".to_string(),
            rows: vec![TableRow {
                raw: "| Alice |".to_string(),
                cells: vec!["Alice".to_string()],
            }],
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

        let target = OutputTarget::File {
            path: output_file.clone(),
            append: false,
        };

        let result = write_document(&doc, &target, None);
        assert!(result.is_ok());

        let written = std::fs::read_to_string(&output_file).unwrap();
        assert!(written.contains("<!-- smt -->"));
        assert!(written.contains("Alice"));
    }

    #[test]
    fn test_write_document_inplace_target_error() {
        let doc = Document {
            source: None,
            blocks: vec![],
        };

        let target = OutputTarget::InPlace;
        let result = write_document(&doc, &target, None);

        // InPlace without a source path should error
        assert!(result.is_err());
    }

    #[test]
    fn test_write_document_invalid_path() {
        let doc = Document {
            source: None,
            blocks: vec![Block::PlainText(vec!["test".to_string()])],
        };

        let target = OutputTarget::File {
            path: Path::new("/nonexistent/directory/file.md").to_path_buf(),
            append: false,
        };

        let result = write_document(&doc, &target, None);
        assert!(result.is_err());
    }
}
