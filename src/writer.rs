//! Render [`Document`] strings and deliver them — stdout,
//! [`crate::cli::OutputTarget`] routing, or inplace atomic writes.
//!
//! Entry points: [`render_document`], [`write_document`],
//! [`write_documents_in_place_atomic`].
// # ============================================================================ PHASE 5: Writer Module — Output Handling
//
// Responsibility: Render Document to string and write to various targets (stdout,
// file, or in-place with atomic writes).
//
// Key requirements:
//
// 1. render_document: Join all blocks + tables into a single string
//
// 2. write_document: Route to stdout/file/in-place with error handling
//
// 3. Atomic writes for -i mode using tempfile::NamedTempFile
//
// 4. # Preserve all non-sorted content exactly as-is
use crate::cli::OutputTarget;
use crate::error::SmtError;
use crate::parser::Document;
use crate::parser::{Block, LineEnding, TextLine};
#[cfg(test)]
use crate::parser::{Table, TableRow};
use std::fs;
use std::fs::OpenOptions;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use tempfile::NamedTempFile;

// # ============================================================================ TASK 5.1: Render Document to String
/// Render a Document to a complete markdown string.
///
/// Algorithm:
///
/// 1. Iterate through all blocks
///
/// 2. For PlainText blocks: join lines with newlines
///
/// 3. For SortedTable blocks: render table (header, separator, sorted rows)
///
/// 4. Join all rendered blocks with newlines
///
/// The rendered output preserves all content, with sorted tables updated.
pub fn render_document(doc: &Document, line_ending: Option<LineEnding>) -> String {
    let mut remaining = doc
        .blocks
        .iter()
        .map(|block| match block {
            Block::PlainText(lines) => lines.len(),
            Block::SortedTable {
                table,
                blank_lines_after_comment,
                ..
            } => 3 + blank_lines_after_comment.len() + table.rows.len(),
        })
        .sum::<usize>();
    let mut output = String::new();
    let mut append_line = |line: &TextLine, output: &mut String| {
        output.push_str(&line.text);
        remaining -= 1;
        if remaining > 0 {
            output.push_str(
                line_ending
                    .unwrap_or(line.line_ending.unwrap_or(doc.line_ending))
                    .as_str(),
            );
        }
    };

    for block in &doc.blocks {
        match block {
            Block::PlainText(lines) => {
                for line in lines {
                    append_line(line, &mut output);
                }
            }
            Block::SortedTable {
                comment_line,
                table,
                blank_lines_after_comment,
                ..
            } => {
                append_line(comment_line, &mut output);
                for blank_line in blank_lines_after_comment {
                    append_line(blank_line, &mut output);
                }
                append_line(&table.header, &mut output);
                append_line(&table.separator, &mut output);
                for row in &table.rows {
                    append_line(&row.raw, &mut output);
                }
            }
        }
    }
    output
}

fn last_line_ending(doc: &Document) -> Option<LineEnding> {
    for block in doc.blocks.iter().rev() {
        let line = match block {
            Block::PlainText(lines) => lines.last(),
            Block::SortedTable { table, .. } => table
                .rows
                .last()
                .map(|row| &row.raw)
                .or(Some(&table.separator)),
        };
        if let Some(line) = line {
            return line.line_ending;
        }
    }
    None
}

// # ============================================================================ TASK 5.2: Atomic Write Logic for In-Place Mode
/// Write to a file atomically using tempfile.
///
/// Algorithm:
///
/// 1. Create a NamedTempFile in the same directory as the target
///
/// 2. Write content to the temp file
///
/// 3. If write succeeds: persist (rename) temp → target
///
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

    // Persist the temp file to the target path This performs an atomic rename on most
    // systems
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

fn backup_path_for(path: &Path) -> PathBuf {
    let base = PathBuf::from(format!("{}.smt.bak", path.display()));
    if !base.exists() {
        return base;
    }
    for i in 1.. {
        let candidate = PathBuf::from(format!("{}.{}", base.display(), i));
        if !candidate.exists() {
            return candidate;
        }
    }
    unreachable!("infinite loop only if filesystem always returns exists");
}

/// Transactional in-place write across multiple files.
///
/// Guarantee: if this function returns Err, all original files are restored.
pub fn write_documents_in_place_atomic(entries: Vec<(PathBuf, String)>) -> Result<(), SmtError> {
    if entries.is_empty() {
        return Ok(());
    }

    // Prepare: write all new contents to temp files first (no originals touched).
    let mut temps: Vec<(PathBuf, NamedTempFile)> = Vec::with_capacity(entries.len());
    for (path, content) in &entries {
        let dir = path.parent().unwrap_or_else(|| Path::new("."));
        let permissions = fs::metadata(path)
            .map_err(|e| map_io_error(&e, path))?
            .permissions();
        let mut temp_file = NamedTempFile::new_in(dir).map_err(|e| map_io_error(&e, path))?;
        temp_file
            .write_all(content.as_bytes())
            .map_err(|e| map_io_error(&e, path))?;
        temp_file
            .as_file()
            .set_permissions(permissions)
            .map_err(|e| map_io_error(&e, path))?;
        temp_file.flush().map_err(|e| map_io_error(&e, path))?;
        temp_file
            .as_file()
            .sync_all()
            .map_err(|e| map_io_error(&e, path))?;
        temps.push((path.clone(), temp_file));
    }

    // Commit (with rollback):
    //
    // 1. move originals to backups
    //
    // 2. persist temp files into final paths
    let mut backups: Vec<(PathBuf, PathBuf)> = Vec::with_capacity(entries.len());
    let mut persisted: Vec<PathBuf> = Vec::new();
    let rollback = |backups: Vec<(PathBuf, PathBuf)>, persisted: Vec<PathBuf>| {
        // Best-effort: restore originals from backups.
        for path in persisted {
            let _ = fs::remove_file(&path);
        }
        for (path, backup) in backups.into_iter().rev() {
            let _ = fs::remove_file(&path);
            let _ = fs::rename(&backup, &path);
        }
    };
    for (path, _) in &temps {
        let backup = backup_path_for(path);
        fs::rename(path, &backup).map_err(|e| {
            rollback(backups.clone(), persisted.clone());
            map_io_error(&e, path)
        })?;
        backups.push((path.clone(), backup));
    }
    for (path, temp_file) in temps {
        if let Err(e) = temp_file.persist(&path) {
            rollback(backups, persisted);
            return Err(map_io_error(&e.error, &path));
        }
        persisted.push(path);
    }

    // Cleanup backups on success.
    for (_path, backup) in backups {
        let _ = fs::remove_file(backup);
    }
    Ok(())
}

// # ============================================================================ TASK 5.3: Write Document Orchestration
/// Write a rendered document to the specified output target.
///
/// Algorithm:
///
/// 1. Render the document to string
///
/// 2. Match on OutputTarget:
///
///    * Stdout: println!()
///
///    * File { path, append }: write to file (create if new, append if flag set)
///
///    * InPlace: atomic write (temp file + rename)
///
/// 3. Return Ok(()) on success, SmtError on failure
///
/// All errors are mapped to SmtError with source location context.
pub fn write_document(
    doc: &Document,
    target: &OutputTarget,
    source: Option<&Path>,
) -> Result<(), SmtError> {
    let output_path = match target {
        OutputTarget::File { path, .. } => Some(path.as_path()),
        OutputTarget::Stdout | OutputTarget::InPlace => source,
    };
    let line_ending = output_path
        .map(crate::editorconfig::resolve_end_of_line)
        .transpose()?
        .flatten();
    let content = render_document(doc, line_ending);

    match target {
        OutputTarget::Stdout => {
            let stdout_ending = line_ending
                .or_else(|| {
                    doc.trailing_newline
                        .then(|| last_line_ending(doc))
                        .flatten()
                })
                .unwrap_or(LineEnding::Lf);
            let mut stdout = io::stdout().lock();
            stdout
                .write_all(content.as_bytes())
                .and_then(|()| stdout.write_all(stdout_ending.as_str().as_bytes()))
                .map_err(|source| SmtError::Io { source })?;
            Ok(())
        }
        OutputTarget::File { path, append } => {
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
            let path = source.ok_or_else(|| SmtError::Io {
                source: io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "InPlace target requires a source path",
                ),
            })?;
            write_atomic(path, &content)
        }
    }
}

// # ============================================================================ UNIT TESTS
#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::{LineEnding, SortOptions, TextLine};

    // # ======================================================================== TASK 5.1: Render Document Tests
    #[test]
    fn test_render_plain_text_only() {
        let doc = Document {
            source: None,
            blocks: vec![Block::PlainText(vec![
                TextLine::new("# Heading", LineEnding::Lf),
                TextLine::new("Some text", LineEnding::Lf),
            ])],
            line_ending: LineEnding::Lf,
            trailing_newline: false,
        };
        let rendered = render_document(&doc, None);
        assert_eq!(rendered, "# Heading\nSome text");
    }

    #[test]
    fn test_render_empty_document() {
        let doc = Document {
            source: None,
            blocks: vec![],
            line_ending: LineEnding::Lf,
            trailing_newline: false,
        };
        let rendered = render_document(&doc, None);
        assert_eq!(rendered, "");
    }

    #[test]
    fn test_render_table_only() {
        let table = Table {
            start_line: 1,
            header: TextLine::new("| Name | Age |", LineEnding::Lf),
            separator: TextLine::new("| --- | --- |", LineEnding::Lf),
            rows: vec![
                TableRow {
                    raw: TextLine::new("| Alice | 30 |", LineEnding::Lf),
                    cells: vec!["Alice".to_string(), "30".to_string()],
                },
                TableRow {
                    raw: TextLine::new("| Bob | 25 |", LineEnding::Lf),
                    cells: vec!["Bob".to_string(), "25".to_string()],
                },
            ],
            column_count: 2,
        };
        let doc = Document {
            source: None,
            blocks: vec![Block::SortedTable {
                comment_line: TextLine::new("<!-- smt -->", LineEnding::Lf),
                comment_line_number: 0,
                options: SortOptions::default(),
                table,
                blank_lines_after_comment: Vec::new(),
            }],
            line_ending: LineEnding::Lf,
            trailing_newline: false,
        };
        let rendered = render_document(&doc, None);
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
            header: TextLine::new("| A | B |", LineEnding::Lf),
            separator: TextLine::new("| - | - |", LineEnding::Lf),
            rows: vec![TableRow {
                raw: TextLine::new("| 1 | 2 |", LineEnding::Lf),
                cells: vec!["1".to_string(), "2".to_string()],
            }],
            column_count: 2,
        };
        let doc = Document {
            source: None,
            blocks: vec![
                Block::PlainText(vec![TextLine::new("# Title", LineEnding::Lf)]),
                Block::SortedTable {
                    comment_line: TextLine::new("<!-- smt -->", LineEnding::Lf),
                    comment_line_number: 2,
                    options: SortOptions::default(),
                    table,
                    blank_lines_after_comment: Vec::new(),
                },
                Block::PlainText(vec![TextLine::new("Done.", LineEnding::Lf)]),
            ],
            line_ending: LineEnding::Lf,
            trailing_newline: false,
        };
        let rendered = render_document(&doc, None);
        assert!(rendered.contains("# Title"));
        assert!(rendered.contains("<!-- smt -->"));
        assert!(rendered.contains("Done."));
    }

    #[test]
    fn test_render_multiple_tables() {
        let table1 = Table {
            start_line: 1,
            header: TextLine::new("| Col |", LineEnding::Lf),
            separator: TextLine::new("| --- |", LineEnding::Lf),
            rows: vec![TableRow {
                raw: TextLine::new("| A |", LineEnding::Lf),
                cells: vec!["A".to_string()],
            }],
            column_count: 1,
        };
        let table2 = Table {
            start_line: 5,
            header: TextLine::new("| X |", LineEnding::Lf),
            separator: TextLine::new("| - |", LineEnding::Lf),
            rows: vec![TableRow {
                raw: TextLine::new("| Y |", LineEnding::Lf),
                cells: vec!["Y".to_string()],
            }],
            column_count: 1,
        };
        let doc = Document {
            source: None,
            blocks: vec![
                Block::SortedTable {
                    comment_line: TextLine::new("<!-- smt -->", LineEnding::Lf),
                    comment_line_number: 0,
                    options: SortOptions::default(),
                    table: table1,
                    blank_lines_after_comment: Vec::new(),
                },
                Block::PlainText(vec![TextLine::new("---", LineEnding::Lf)]),
                Block::SortedTable {
                    comment_line: TextLine::new("<!-- smt -->", LineEnding::Lf),
                    comment_line_number: 4,
                    options: SortOptions::default(),
                    table: table2,
                    blank_lines_after_comment: Vec::new(),
                },
            ],
            line_ending: LineEnding::Lf,
            trailing_newline: false,
        };
        let rendered = render_document(&doc, None);
        let lines: Vec<&str> = rendered.lines().collect();

        // Should have: comment, header, separator, row, "---", comment, header,
        // separator, row
        assert!(lines.len() >= 9);
        assert_eq!(lines[0], "<!-- smt -->");
        assert_eq!(lines[1], "| Col |");
    }

    #[test]
    fn test_render_preserves_content() {
        let doc = Document {
            source: None,
            blocks: vec![
                Block::PlainText(vec![
                    TextLine::new("Line 1", LineEnding::Lf),
                    TextLine::new("Line 2", LineEnding::Lf),
                ]),
                Block::PlainText(vec![TextLine::new("Line 3", LineEnding::Lf)]),
            ],
            line_ending: LineEnding::Lf,
            trailing_newline: false,
        };
        let rendered = render_document(&doc, None);
        assert_eq!(rendered, "Line 1\nLine 2\nLine 3");
    }

    #[test]
    fn test_render_empty_plaintext() {
        let doc = Document {
            source: None,
            blocks: vec![Block::PlainText(vec![])],
            line_ending: LineEnding::Lf,
            trailing_newline: false,
        };
        let rendered = render_document(&doc, None);
        assert_eq!(rendered, "");
    }

    // # ======================================================================== TASK 5.2: Atomic Write Tests
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

    // # ======================================================================== TASK 5.3: Write Document Tests
    #[test]
    fn test_write_document_stdout() {
        let doc = Document {
            source: None,
            blocks: vec![Block::PlainText(vec![TextLine::new(
                "# Test",
                LineEnding::Lf,
            )])],
            line_ending: LineEnding::Lf,
            trailing_newline: false,
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
            blocks: vec![Block::PlainText(vec![TextLine::new(
                "# Test",
                LineEnding::Lf,
            )])],
            line_ending: LineEnding::Lf,
            trailing_newline: false,
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
            blocks: vec![Block::PlainText(vec![TextLine::new(
                "Appended",
                LineEnding::Lf,
            )])],
            line_ending: LineEnding::Lf,
            trailing_newline: false,
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
            blocks: vec![Block::PlainText(vec![TextLine::new(
                "New file",
                LineEnding::Lf,
            )])],
            line_ending: LineEnding::Lf,
            trailing_newline: false,
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
            header: TextLine::new("| Name |", LineEnding::Lf),
            separator: TextLine::new("| --- |", LineEnding::Lf),
            rows: vec![TableRow {
                raw: TextLine::new("| Alice |", LineEnding::Lf),
                cells: vec!["Alice".to_string()],
            }],
            column_count: 1,
        };
        let doc = Document {
            source: None,
            blocks: vec![Block::SortedTable {
                comment_line: TextLine::new("<!-- smt -->", LineEnding::Lf),
                comment_line_number: 0,
                options: SortOptions::default(),
                table,
                blank_lines_after_comment: Vec::new(),
            }],
            line_ending: LineEnding::Lf,
            trailing_newline: false,
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
            line_ending: LineEnding::Lf,
            trailing_newline: false,
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
            blocks: vec![Block::PlainText(vec![TextLine::new(
                "test",
                LineEnding::Lf,
            )])],
            line_ending: LineEnding::Lf,
            trailing_newline: false,
        };
        let target = OutputTarget::File {
            path: Path::new("/nonexistent/directory/file.md").to_path_buf(),
            append: false,
        };
        let result = write_document(&doc, &target, None);
        assert!(result.is_err());
    }
}
