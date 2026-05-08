use crate::error::{SmtError, SourceLocation};
use std::path::PathBuf;

// ============================================================================
// TASK 3.1: Parser Data Structures
// ============================================================================

/// A parsed markdown document
#[derive(Debug, Clone)]
pub struct Document {
    pub source: Option<PathBuf>,
    pub blocks: Vec<Block>,
}

/// A block within the document
#[derive(Debug, Clone)]
pub enum Block {
    PlainText(Vec<String>),
    SortedTable {
        comment_line: String,
        comment_line_number: usize,
        options: SortOptions,
        table: Table,
        blank_lines_after_comment: Vec<String>,
    },
}

/// Parsed options from an `<!-- smt ... -->` comment
#[derive(Debug, Clone, PartialEq)]
pub struct SortOptions {
    pub column: usize,
    pub order: SortOrder,
    pub case: CaseSensitivity,
    pub sort_type: SortType,
}

impl Default for SortOptions {
    fn default() -> Self {
        Self {
            column: 1,
            order: SortOrder::Asc,
            case: CaseSensitivity::Sensitive,
            sort_type: SortType::Numeric,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SortOrder {
    Asc,
    Desc,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CaseSensitivity {
    Sensitive,
    Insensitive,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SortType {
    Numeric,
    Lexicographic,
}

/// A markdown table
#[derive(Debug, Clone)]
pub struct Table {
    pub start_line: usize,
    pub header: String,
    pub separator: String,
    pub rows: Vec<TableRow>,
    pub column_count: usize,
}

/// A single data row in a table
#[derive(Debug, Clone)]
pub struct TableRow {
    pub raw: String,
    pub cells: Vec<String>,
}

// ============================================================================
// TASK 3.2: Comment Parsing (11-Step Algorithm)
// ============================================================================

/// Parse an SMT comment and extract sort options.
///
/// Algorithm (11 steps):
/// 1. Strip leading/trailing whitespace
/// 2. Strip "<!--" prefix and "-->" suffix
/// 3. Strip "smt" prefix
/// 4. If empty, return default SortOptions
/// 5. Split by whitespace into tokens
///
/// 6-9. For each token: parse key=value, validate, parse value
/// 10. Collect parsed options
/// 11. Return SortOptions with parsed + defaults
pub fn parse_sort_options(
    comment_text: &str,
    line_num: usize,
    source: Option<PathBuf>,
) -> Result<SortOptions, SmtError> {
    let source_loc = SourceLocation(source.clone());

    // Step 1: Strip leading/trailing whitespace
    let trimmed = comment_text.trim();

    // Step 2: Strip "<!--" prefix and "-->" suffix
    let no_prefix = trimmed
        .strip_prefix("<!--")
        .ok_or_else(|| SmtError::InvalidOptionValue {
            path: source_loc.clone(),
            line: line_num,
            key: "comment".to_string(),
            value: comment_text.to_string(),
            expected: "<!-- smt ... -->".to_string(),
        })?
        .trim();

    let no_suffix = no_prefix
        .strip_suffix("-->")
        .ok_or_else(|| SmtError::InvalidOptionValue {
            path: source_loc.clone(),
            line: line_num,
            key: "comment".to_string(),
            value: comment_text.to_string(),
            expected: "<!-- smt ... -->".to_string(),
        })?
        .trim();

    // Step 3: Strip "smt" prefix
    let after_smt = no_suffix
        .strip_prefix("smt")
        .ok_or_else(|| SmtError::InvalidOptionValue {
            path: source_loc.clone(),
            line: line_num,
            key: "comment".to_string(),
            value: comment_text.to_string(),
            expected: "<!-- smt ... -->".to_string(),
        })?
        .trim();

    // Step 4: If empty, return default SortOptions
    if after_smt.is_empty() {
        return Ok(SortOptions::default());
    }

    // Step 5: Split by whitespace into tokens
    let tokens: Vec<&str> = after_smt.split_whitespace().collect();

    // Steps 6-10: Parse each token
    let mut options = SortOptions::default();

    for token in tokens {
        // Step 6a: Split on '='
        let parts: Vec<&str> = token.split('=').collect();
        if parts.len() != 2 {
            return Err(SmtError::InvalidOptionValue {
                path: source_loc.clone(),
                line: line_num,
                key: token.to_string(),
                value: token.to_string(),
                expected: "key=value format".to_string(),
            });
        }

        let key = parts[0];
        let value = parts[1];

        // Step 6b: Match key against known keys
        match key {
            "column" => {
                // Step 6c-f: Parse column value
                let col: usize = value.parse().map_err(|_| SmtError::ColumnNotInteger {
                    path: source_loc.clone(),
                    line: line_num,
                })?;

                if col == 0 {
                    return Err(SmtError::ColumnZero {
                        path: source_loc.clone(),
                        line: line_num,
                    });
                }

                options.column = col;
            }
            "order" => {
                options.order = match value {
                    "asc" => SortOrder::Asc,
                    "desc" => SortOrder::Desc,
                    _ => {
                        return Err(SmtError::InvalidOptionValue {
                            path: source_loc.clone(),
                            line: line_num,
                            key: "order".to_string(),
                            value: value.to_string(),
                            expected: "asc or desc".to_string(),
                        })
                    }
                };
            }
            "case" => {
                options.case = match value {
                    "sensitive" => CaseSensitivity::Sensitive,
                    "insensitive" => CaseSensitivity::Insensitive,
                    _ => {
                        return Err(SmtError::InvalidOptionValue {
                            path: source_loc.clone(),
                            line: line_num,
                            key: "case".to_string(),
                            value: value.to_string(),
                            expected: "sensitive or insensitive".to_string(),
                        })
                    }
                };
            }
            "type" => {
                options.sort_type = match value {
                    "numeric" => SortType::Numeric,
                    "lexicographic" => SortType::Lexicographic,
                    _ => {
                        return Err(SmtError::InvalidOptionValue {
                            path: source_loc.clone(),
                            line: line_num,
                            key: "type".to_string(),
                            value: value.to_string(),
                            expected: "numeric or lexicographic".to_string(),
                        })
                    }
                };
            }
            _ => {
                // Step 6d: Unknown key
                return Err(SmtError::UnknownOption {
                    path: source_loc.clone(),
                    line: line_num,
                    key: key.to_string(),
                });
            }
        }
    }

    // Step 11: Return SortOptions with parsed values + defaults
    Ok(options)
}

// ============================================================================
// TASK 3.3: Parser State Machine & Line Classification
// ============================================================================

/// Check if a line is an SMT comment: `^\s*<!--\s+smt(\s+.*)?\s*-->\s*$`
fn is_smt_comment(line: &str) -> bool {
    let trimmed = line.trim();

    // Must start with <!-- and end with -->
    if !trimmed.starts_with("<!--") || !trimmed.ends_with("-->") {
        return false;
    }

    // Extract content between <!-- and -->
    if let Some(content) = trimmed.strip_prefix("<!--") {
        if let Some(content) = content.strip_suffix("-->") {
            let content = content.trim();
            // Must start with "smt" keyword
            return content.starts_with("smt")
                && (content.len() == 3 || content.chars().nth(3).is_some_and(char::is_whitespace));
        }
    }

    false
}

/// Check if a line is empty or whitespace-only
fn is_empty_line(line: &str) -> bool {
    line.trim().is_empty()
}

/// Check if a line is a table row: `^\s*\|.*\|\s*$`
fn is_table_row(line: &str) -> bool {
    let trimmed = line.trim();
    trimmed.starts_with('|') && trimmed.ends_with('|')
}

/// Check if a line is a separator row (all cells are dashes/colons)
fn is_separator_row(line: &str) -> bool {
    if !is_table_row(line) {
        return false;
    }

    let cells = extract_cells(line);
    !cells.is_empty() && cells.iter().all(|cell| is_valid_separator_cell(cell))
}

/// Extract cells from a table row
fn extract_cells(line: &str) -> Vec<String> {
    let parts: Vec<&str> = line.split('|').collect();

    // Skip first empty (before |) and last empty (after |)
    if parts.len() < 2 {
        return Vec::new();
    }

    parts[1..parts.len() - 1]
        .iter()
        .map(|s| s.trim().to_string())
        .collect()
}

/// Check if a cell is a valid separator: `^:?-+:?$`
fn is_valid_separator_cell(cell: &str) -> bool {
    if cell.is_empty() {
        return false;
    }

    let trimmed = cell.trim();

    // Strip leading colon if present
    let after_leading = if let Some(stripped) = trimmed.strip_prefix(':') {
        stripped
    } else {
        trimmed
    };

    // Strip trailing colon if present
    let after_trailing = if let Some(stripped) = after_leading.strip_suffix(':') {
        stripped
    } else {
        after_leading
    };

    // Must have at least one dash
    !after_trailing.is_empty() && after_trailing.chars().all(|c| c == '-')
}

/// Parser state machine
#[derive(Debug, Clone, Copy, PartialEq)]
enum ParserState {
    Normal,
    ExpectTable,
    ExpectSep,
    ReadingRows,
}

/// Parse markdown content into a Document
pub fn parse(content: &str, source: Option<PathBuf>) -> Result<Document, SmtError> {
    let source_loc = SourceLocation(source.clone());
    let lines: Vec<&str> = content.lines().collect();

    let mut blocks = Vec::new();
    let mut state = ParserState::Normal;
    let mut current_plain_text = Vec::new();
    let mut blank_lines_after_comment = Vec::new();
    let mut pending_comment_line = String::new();
    let mut pending_comment_line_num = 0;
    let mut pending_options = SortOptions::default();
    let mut table_header = String::new();
    let mut table_separator = String::new();
    let mut table_rows = Vec::new();
    let mut table_start_line = 0;
    let mut previous_comment_line = 0;

    for (line_num, line) in lines.iter().enumerate() {
        let line_num_1based = line_num + 1;

        match state {
            ParserState::Normal => {
                if is_smt_comment(line) {
                    // Found an SMT comment - transition to ExpectTable
                    if !current_plain_text.is_empty() {
                        blocks.push(Block::PlainText(current_plain_text.clone()));
                        current_plain_text.clear();
                    }

                    // Check for duplicate comment (if we just finalized a table)
                    if previous_comment_line != 0 && line_num_1based == previous_comment_line + 2 {
                        // This is a comment right after a previous table's last row
                        // Not necessarily duplicate, just consecutive
                    }

                    pending_comment_line = line.to_string();
                    pending_comment_line_num = line_num_1based;

                    // Parse options from comment
                    pending_options = parse_sort_options(line, line_num_1based, source.clone())?;

                    state = ParserState::ExpectTable;
                } else {
                    // Regular line - add to plain text block
                    current_plain_text.push(line.to_string());
                }
            }

            ParserState::ExpectTable => {
                if is_empty_line(line) {
                    // Blank line after comment - track it separately
                    blank_lines_after_comment.push(line.to_string());
                } else if is_table_row(line) {
                    // Found table header - transition to ExpectSep
                    table_header = line.to_string();
                    table_start_line = line_num_1based;
                    state = ParserState::ExpectSep;
                } else {
                    // Error: comment followed by non-table, non-empty line
                    return Err(SmtError::CommentWithoutTable {
                        path: source_loc,
                        line: pending_comment_line_num,
                    });
                }
            }

            ParserState::ExpectSep => {
                if is_separator_row(line) {
                    // Found separator - transition to ReadingRows
                    table_separator = line.to_string();
                    table_rows.clear();
                    state = ParserState::ReadingRows;
                } else {
                    // Error: malformed table
                    return Err(SmtError::MalformedTable {
                        path: source_loc,
                        line: table_start_line,
                    });
                }
            }

            ParserState::ReadingRows => {
                if is_table_row(line) {
                    // Collect data row
                    let cells = extract_cells(line);
                    let expected_col_count = extract_cells(&table_header).len();

                    if cells.len() != expected_col_count {
                        return Err(SmtError::MalformedTable {
                            path: source_loc,
                            line: table_start_line,
                        });
                    }

                    table_rows.push(TableRow {
                        raw: line.to_string(),
                        cells,
                    });
                } else {
                    // End of table - finalize and emit
                    finalize_table(
                        &mut blocks,
                        &mut current_plain_text,
                        TableContext {
                            header: &table_header,
                            separator: &table_separator,
                            rows: &table_rows,
                            comment_line: &pending_comment_line,
                            comment_line_num: pending_comment_line_num,
                            options: &pending_options,
                            start_line: table_start_line,
                            blank_lines_after_comment: &blank_lines_after_comment,
                        },
                        &mut previous_comment_line,
                        &source_loc,
                    )?;

                    current_plain_text.push(line.to_string());
                    blank_lines_after_comment.clear();
                    state = ParserState::Normal;
                }
            }
        }
    }

    // Finalize any pending state
    match state {
        ParserState::Normal => {
            if !current_plain_text.is_empty() {
                blocks.push(Block::PlainText(current_plain_text));
            }
        }
        ParserState::ExpectTable => {
            // Error: comment without table at end of file
            return Err(SmtError::CommentWithoutTable {
                path: source_loc,
                line: pending_comment_line_num,
            });
        }
        ParserState::ExpectSep => {
            // Error: malformed table at end of file
            return Err(SmtError::MalformedTable {
                path: source_loc,
                line: table_start_line,
            });
        }
        ParserState::ReadingRows => {
            // End of table at EOF - finalize
            finalize_table(
                &mut blocks,
                &mut current_plain_text,
                TableContext {
                    header: &table_header,
                    separator: &table_separator,
                    rows: &table_rows,
                    comment_line: &pending_comment_line,
                    comment_line_num: pending_comment_line_num,
                    options: &pending_options,
                    start_line: table_start_line,
                    blank_lines_after_comment: &blank_lines_after_comment,
                },
                &mut previous_comment_line,
                &source_loc,
            )?;
        }
    }

    Ok(Document { source, blocks })
}

/// Table finalization context
struct TableContext<'a> {
    header: &'a str,
    separator: &'a str,
    rows: &'a [TableRow],
    comment_line: &'a str,
    comment_line_num: usize,
    options: &'a SortOptions,
    start_line: usize,
    blank_lines_after_comment: &'a [String],
}

/// Helper function to finalize a table block
fn finalize_table(
    blocks: &mut Vec<Block>,
    current_plain_text: &mut Vec<String>,
    context: TableContext,
    previous_comment_line: &mut usize,
    source_loc: &SourceLocation,
) -> Result<(), SmtError> {
    let header_cells = extract_cells(context.header);
    let column_count = header_cells.len();

    // Validate column is in range
    if context.options.column > column_count {
        return Err(SmtError::ColumnOutOfRange {
            path: source_loc.clone(),
            line: context.comment_line_num,
            column: context.options.column,
            actual: column_count,
        });
    }

    let table = Table {
        start_line: context.start_line,
        header: context.header.to_string(),
        separator: context.separator.to_string(),
        rows: context.rows.to_vec(),
        column_count,
    };

    blocks.push(Block::SortedTable {
        comment_line: context.comment_line.to_string(),
        comment_line_number: context.comment_line_num,
        options: context.options.clone(),
        table,
        blank_lines_after_comment: context.blank_lines_after_comment.to_vec(),
    });

    *previous_comment_line = context.comment_line_num;
    current_plain_text.clear();

    Ok(())
}

// ============================================================================
// TASK 3.4: Comprehensive Parser Unit Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    // Test 3.1: Default options
    #[test]
    fn test_parse_options_default() {
        let opts = parse_sort_options("<!-- smt -->", 1, None).unwrap();
        assert_eq!(opts.column, 1);
        assert_eq!(opts.order, SortOrder::Asc);
        assert_eq!(opts.case, CaseSensitivity::Sensitive);
        assert_eq!(opts.sort_type, SortType::Numeric);
    }

    // Test 3.2: Single option - column
    #[test]
    fn test_parse_options_column() {
        let opts = parse_sort_options("<!-- smt column=2 -->", 1, None).unwrap();
        assert_eq!(opts.column, 2);
    }

    // Test 3.3: Single option - order desc
    #[test]
    fn test_parse_options_order_desc() {
        let opts = parse_sort_options("<!-- smt order=desc -->", 1, None).unwrap();
        assert_eq!(opts.order, SortOrder::Desc);
    }

    // Test 3.4: Single option - case insensitive
    #[test]
    fn test_parse_options_case_insensitive() {
        let opts = parse_sort_options("<!-- smt case=insensitive -->", 1, None).unwrap();
        assert_eq!(opts.case, CaseSensitivity::Insensitive);
    }

    // Test 3.5: Single option - type lexicographic
    #[test]
    fn test_parse_options_type_lexicographic() {
        let opts = parse_sort_options("<!-- smt type=lexicographic -->", 1, None).unwrap();
        assert_eq!(opts.sort_type, SortType::Lexicographic);
    }

    // Test 3.6: Multiple options
    #[test]
    fn test_parse_options_multiple() {
        let opts = parse_sort_options(
            "<!-- smt column=3 order=desc case=insensitive type=lexicographic -->",
            1,
            None,
        )
        .unwrap();
        assert_eq!(opts.column, 3);
        assert_eq!(opts.order, SortOrder::Desc);
        assert_eq!(opts.case, CaseSensitivity::Insensitive);
        assert_eq!(opts.sort_type, SortType::Lexicographic);
    }

    // Test 3.7: Unknown option key
    #[test]
    fn test_parse_options_unknown_key() {
        let result = parse_sort_options("<!-- smt colum=2 -->", 1, None);
        assert!(matches!(result, Err(SmtError::UnknownOption { .. })));
    }

    // Test 3.8: Invalid order value
    #[test]
    fn test_parse_options_invalid_order() {
        let result = parse_sort_options("<!-- smt order=ascending -->", 1, None);
        assert!(matches!(result, Err(SmtError::InvalidOptionValue { .. })));
    }

    // Test 3.9: Invalid type value
    #[test]
    fn test_parse_options_invalid_type() {
        let result = parse_sort_options("<!-- smt type=foo -->", 1, None);
        assert!(matches!(result, Err(SmtError::InvalidOptionValue { .. })));
    }

    // Test 3.10: Column = 0
    #[test]
    fn test_parse_options_column_zero() {
        let result = parse_sort_options("<!-- smt column=0 -->", 1, None);
        assert!(matches!(result, Err(SmtError::ColumnZero { .. })));
    }

    // Test 3.11: Column not integer
    #[test]
    fn test_parse_options_column_not_integer() {
        let result = parse_sort_options("<!-- smt column=abc -->", 1, None);
        assert!(matches!(result, Err(SmtError::ColumnNotInteger { .. })));
    }

    // Test 3.12: Valid separator row variations
    #[test]
    fn test_separator_row_variations() {
        assert!(is_separator_row("| --- | --- | --- |"));
        assert!(is_separator_row("| :--- | :--- | :--- |"));
        assert!(is_separator_row("| ---: | ---: | ---: |"));
        assert!(is_separator_row("| :---: | :---: | :---: |"));
    }

    // Test 3.13: Invalid separator row
    #[test]
    fn test_separator_row_invalid() {
        assert!(!is_separator_row("| header | column | data |"));
        // Single column separator is VALID
        assert!(is_separator_row("| --- |"));
    }

    // Test 3.14: Table row detection
    #[test]
    fn test_is_table_row() {
        assert!(is_table_row("| col1 | col2 | col3 |"));
        assert!(is_table_row("  | col1 | col2 |  ")); // With whitespace
        assert!(!is_table_row("no pipes here"));
        assert!(!is_table_row("| col1 col2 col3")); // No trailing pipe
    }

    // Test 3.15: SMT comment detection
    #[test]
    fn test_is_smt_comment() {
        assert!(is_smt_comment("<!-- smt -->"));
        assert!(is_smt_comment("  <!-- smt -->  "));
        assert!(is_smt_comment("<!-- smt column=2 -->"));
        assert!(!is_smt_comment("<!-- other comment -->"));
        assert!(!is_smt_comment("not a comment"));
    }

    // Test 3.16: Simple valid table
    #[test]
    fn test_parse_simple_table() {
        let markdown = "<!-- smt -->\n| Name | Age |\n| --- | --- |\n| Alice | 30 |\n| Bob | 25 |";
        let doc = parse(markdown, None).unwrap();
        assert_eq!(doc.blocks.len(), 1);
        match &doc.blocks[0] {
            Block::SortedTable {
                comment_line_number,
                table,
                ..
            } => {
                assert_eq!(*comment_line_number, 1);
                assert_eq!(table.rows.len(), 2);
                assert_eq!(table.column_count, 2);
            }
            _ => panic!("Expected SortedTable"),
        }
    }

    // Test 3.17: Table without comment
    #[test]
    fn test_parse_unmarked_table() {
        let markdown = "| Name | Age |\n| --- | --- |\n| Alice | 30 |";
        let doc = parse(markdown, None).unwrap();
        // Unmarked table should be in plain text block
        assert_eq!(doc.blocks.len(), 1);
        match &doc.blocks[0] {
            Block::PlainText(lines) => {
                assert_eq!(lines.len(), 3);
            }
            _ => panic!("Expected PlainText"),
        }
    }

    // Test 3.18: Comment without table (error)
    #[test]
    fn test_parse_comment_without_table() {
        let markdown = "<!-- smt -->\nSome text";
        let result = parse(markdown, None);
        assert!(matches!(result, Err(SmtError::CommentWithoutTable { .. })));
    }

    // Test 3.19: Column out of range (error)
    #[test]
    fn test_parse_column_out_of_range() {
        let markdown = "<!-- smt column=5 -->\n| Name | Age |\n| --- | --- |\n| Alice | 30 |";
        let result = parse(markdown, None);
        assert!(matches!(result, Err(SmtError::ColumnOutOfRange { .. })));
    }

    // Test 3.20: Malformed table - missing separator
    #[test]
    fn test_parse_malformed_table_no_separator() {
        let markdown = "<!-- smt -->\n| Name | Age |\n| Alice | 30 |";
        let result = parse(markdown, None);
        assert!(matches!(result, Err(SmtError::MalformedTable { .. })));
    }

    // Test 3.21: Multiple tables
    #[test]
    fn test_parse_multiple_tables() {
        let markdown =
            "<!-- smt -->\n| A |\n| --- |\n| 1 |\ntext\n<!-- smt -->\n| B |\n| --- |\n| 2 |";
        let doc = parse(markdown, None).unwrap();
        // Should have: SortedTable, PlainText, SortedTable
        assert_eq!(doc.blocks.len(), 3);
    }

    // Test 3.22: Whitespace preservation
    #[test]
    fn test_parse_whitespace_preservation() {
        let markdown = "<!-- smt -->\n| Name | Age |\n| --- | --- |\n| Alice | 30 |";
        let doc = parse(markdown, None).unwrap();
        match &doc.blocks[0] {
            Block::SortedTable { table, .. } => {
                assert_eq!(table.rows[0].raw, "| Alice | 30 |");
            }
            _ => panic!("Expected SortedTable"),
        }
    }

    // Test 3.23: Empty table (single header row)
    #[test]
    fn test_parse_empty_table() {
        let markdown = "<!-- smt -->\n| Name | Age |\n| --- | --- |";
        let doc = parse(markdown, None).unwrap();
        match &doc.blocks[0] {
            Block::SortedTable { table, .. } => {
                assert_eq!(table.rows.len(), 0);
            }
            _ => panic!("Expected SortedTable"),
        }
    }

    // Test 3.24: Large table
    #[test]
    fn test_parse_large_table() {
        let mut markdown = String::from("<!-- smt -->\n| ID | Name | Age |\n| --- | --- | --- |\n");
        for i in 1..=100 {
            markdown.push_str(&format!("| {} | Person{} | {} |\n", i, i, 20 + i));
        }
        let doc = parse(&markdown, None).unwrap();
        match &doc.blocks[0] {
            Block::SortedTable { table, .. } => {
                assert_eq!(table.rows.len(), 100);
            }
            _ => panic!("Expected SortedTable"),
        }
    }

    // Test 3.25: Mixed plain text and tables
    #[test]
    fn test_parse_mixed_content() {
        let markdown =
            "# Header\n\nSome text.\n\n<!-- smt -->\n| A |\n| --- |\n| 1 |\n\nMore text.\n";
        let doc = parse(markdown, None).unwrap();
        assert_eq!(doc.blocks.len(), 3); // PlainText, SortedTable, PlainText
    }

    // Test 3.26: Line numbers in errors
    #[test]
    fn test_parse_error_line_numbers() {
        let markdown = "text\ntext\n<!-- smt -->\nMore text";
        let result = parse(markdown, None);
        if let Err(SmtError::CommentWithoutTable { line, .. }) = result {
            assert_eq!(line, 3);
        } else {
            panic!("Expected CommentWithoutTable error");
        }
    }

    // Test 3.27: Multiple marked tables
    #[test]
    fn test_parse_multiple_marked_tables() {
        let markdown = "<!-- smt -->\n| A |\n| --- |\n| 1 |\n\n<!-- smt -->\n| B |\n| --- |\n| 2 |";
        let doc = parse(markdown, None).unwrap();
        // Both should parse successfully
        assert_eq!(doc.blocks.len(), 3);
    }

    #[test]
    fn test_parse_fixture_case_insensitive_expected() {
        let markdown =
            fs::read_to_string("tests/fixtures/expected/case_insensitive.expected.md").unwrap();
        let doc = parse(&markdown, Some(PathBuf::from("case_insensitive.expected.md"))).unwrap();
        assert!(!doc.blocks.is_empty());
    }
}
