// ============================================================================
// PHASE 4: Sorter Module — Sorting Logic
// ============================================================================
//
// Responsibility: Sort table rows according to SortOptions, support numeric/
// lexicographic comparison, case sensitivity, and stable sort guarantee.
//
// Key requirements:
// 1. MUST use stable sort (sort_by, never sort_unstable_by)
// 2. Numeric mode: parse as f64, non-numeric goes after numeric (stable)
// 3. Lexicographic: support case-sensitive/insensitive comparison
// 4. Check mode: clone, sort, compare with original
// ============================================================================

use crate::error::SmtError;
#[cfg(test)]
use crate::parser::TableRow;
use crate::parser::{Block, CaseSensitivity, Document, SortOptions, SortOrder, SortType, Table};
use std::cmp::Ordering;
use std::path::PathBuf;

// ============================================================================
// TASK 4.1: Data Structures & Comparators
// ============================================================================

/// Result of checking if a table is sorted
#[derive(Debug, Clone)]
pub struct CheckResult {
    pub source: Option<PathBuf>,
    pub comment_line: usize,
    pub table_start_line: usize,
    pub is_sorted: bool,
}

/// Check all marked tables in a document and return per-table results.
///
/// This is used by `--check` mode to report unsorted locations without
/// modifying the document.
pub fn check_document(doc: &Document) -> Vec<CheckResult> {
    let mut results = Vec::new();

    for block in &doc.blocks {
        if let Block::SortedTable {
            comment_line_number,
            table,
            options,
            ..
        } = block
        {
            let is_sorted = is_table_sorted(table, options);
            results.push(CheckResult {
                source: doc.source.clone(),
                comment_line: *comment_line_number,
                table_start_line: table.start_line,
                is_sorted,
            });
        }
    }

    results
}

/// Compare two strings as numbers (f64), with fallback to lexicographic.
///
/// Algorithm:
/// - Try to parse both as f64
/// - If both numeric: compare as floats (handle NaN via partial_cmp)
/// - If a numeric, b non-numeric: a < b (numbers come first)
/// - If a non-numeric, b numeric: a > b
/// - If both non-numeric: fallback to lexicographic comparison
///
/// This ensures numeric values sort before non-numeric, and among non-numeric
/// values, stability is preserved via lexicographic comparison.
fn compare_numeric(a: &str, b: &str, case: CaseSensitivity) -> Ordering {
    let a_trimmed = a.trim();
    let b_trimmed = b.trim();

    let a_num = a_trimmed.parse::<f64>().ok();
    let b_num = b_trimmed.parse::<f64>().ok();

    match (a_num, b_num) {
        // Both numeric: compare as floats
        (Some(an), Some(bn)) => {
            // Use partial_cmp for f64, which handles NaN correctly
            an.partial_cmp(&bn).unwrap_or(Ordering::Equal)
        }
        // a numeric, b non-numeric: a comes first
        (Some(_), None) => Ordering::Less,
        // a non-numeric, b numeric: b comes first
        (None, Some(_)) => Ordering::Greater,
        // Both non-numeric: use lexicographic comparison
        (None, None) => compare_lexicographic(a, b, case),
    }
}

/// Compare two strings lexicographically with optional case-insensitivity.
///
/// Algorithm:
/// - If case-insensitive: convert both to lowercase, then compare
/// - If case-sensitive: compare as-is
/// - Use standard Rust string ordering
fn compare_lexicographic(a: &str, b: &str, case: CaseSensitivity) -> Ordering {
    match case {
        CaseSensitivity::Insensitive => {
            let a_lower = a.to_lowercase();
            let b_lower = b.to_lowercase();
            a_lower.cmp(&b_lower)
        }
        CaseSensitivity::Sensitive => a.cmp(b),
    }
}

// ============================================================================
// TASK 4.2: Sort Orchestration
// ============================================================================

/// Sort a single table in-place according to the given options.
///
/// This function modifies the table's rows directly using stable sort.
/// The sort column is specified in options (1-based) and is converted to 0-based.
///
/// Precondition: column must be in range (parser validates this)
pub fn sort_table(table: &mut Table, options: &SortOptions) -> Result<(), SmtError> {
    // Convert 1-based column index to 0-based
    let col_idx = options.column - 1;

    // Safety: parser validates column is in range, but we double-check
    if col_idx >= table.column_count {
        return Err(SmtError::ColumnOutOfRange {
            path: crate::error::SourceLocation(None),
            line: table.start_line,
            column: options.column,
            actual: table.column_count,
        });
    }

    // Stable sort using sort_by
    // This is CRITICAL: we MUST use sort_by, never sort_unstable_by
    table.rows.sort_by(|a, b| {
        // Extract the sort key from both rows
        let a_key = &a.cells[col_idx];
        let b_key = &b.cells[col_idx];

        // Apply the appropriate comparator based on sort type
        let cmp = match options.sort_type {
            SortType::Numeric => compare_numeric(a_key, b_key, options.case),
            SortType::Lexicographic => compare_lexicographic(a_key, b_key, options.case),
        };

        // Reverse the comparison if descending
        if options.order == SortOrder::Desc {
            cmp.reverse()
        } else {
            cmp
        }
    });

    Ok(())
}

/// Sort all tables in a document in-place.
///
/// Iterates through all blocks in the document. For each SortedTable block,
/// applies sort_table(). Returns error on first failure (no partial sorts).
pub fn sort_document(doc: &mut Document) -> Result<(), SmtError> {
    for block in &mut doc.blocks {
        if let Block::SortedTable { table, options, .. } = block {
            sort_table(table, options)?;
        }
    }
    Ok(())
}

/// Check if a table is already sorted according to the given options.
///
/// Algorithm:
/// 1. Clone the rows
/// 2. Sort the cloned rows
/// 3. Compare cloned (sorted) with original
/// 4. Return true if no changes (already sorted)
///
/// This avoids modifying the original table while checking.
pub fn is_table_sorted(table: &Table, options: &SortOptions) -> bool {
    // Clone rows for sorting
    let mut sorted_rows = table.rows.clone();

    // Sort the cloned rows (using same logic as sort_table)
    let col_idx = options.column - 1;
    sorted_rows.sort_by(|a, b| {
        let a_key = &a.cells[col_idx];
        let b_key = &b.cells[col_idx];

        let cmp = match options.sort_type {
            SortType::Numeric => compare_numeric(a_key, b_key, options.case),
            SortType::Lexicographic => compare_lexicographic(a_key, b_key, options.case),
        };

        if options.order == SortOrder::Desc {
            cmp.reverse()
        } else {
            cmp
        }
    });

    // Compare original rows with sorted rows
    // Since we're comparing TableRow structs, we compare the raw strings
    // (which should be identical if sorting made no changes)
    table
        .rows
        .iter()
        .zip(sorted_rows.iter())
        .all(|(orig, sorted)| orig.raw == sorted.raw)
}

// ============================================================================
// UNIT TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::LineEnding;

    // ========================================================================
    // TASK 4.1: Comparator Tests
    // ========================================================================

    // Numeric comparisons

    #[test]
    fn test_compare_numeric_both_integers() {
        let cmp = compare_numeric("5", "10", CaseSensitivity::Sensitive);
        assert_eq!(cmp, Ordering::Less);

        let cmp = compare_numeric("10", "5", CaseSensitivity::Sensitive);
        assert_eq!(cmp, Ordering::Greater);

        let cmp = compare_numeric("5", "5", CaseSensitivity::Sensitive);
        assert_eq!(cmp, Ordering::Equal);
    }

    #[test]
    fn test_compare_numeric_floats() {
        let cmp = compare_numeric("3.14", "2.71", CaseSensitivity::Sensitive);
        assert_eq!(cmp, Ordering::Greater);

        let cmp = compare_numeric("1.5", "1.5", CaseSensitivity::Sensitive);
        assert_eq!(cmp, Ordering::Equal);

        let cmp = compare_numeric("0.1", "0.2", CaseSensitivity::Sensitive);
        assert_eq!(cmp, Ordering::Less);
    }

    #[test]
    fn test_compare_numeric_negative_numbers() {
        let cmp = compare_numeric("-5", "-10", CaseSensitivity::Sensitive);
        assert_eq!(cmp, Ordering::Greater);

        let cmp = compare_numeric("-10", "5", CaseSensitivity::Sensitive);
        assert_eq!(cmp, Ordering::Less);

        let cmp = compare_numeric("-5", "-5", CaseSensitivity::Sensitive);
        assert_eq!(cmp, Ordering::Equal);
    }

    #[test]
    fn test_compare_numeric_with_whitespace() {
        let cmp = compare_numeric("  5  ", "10", CaseSensitivity::Sensitive);
        assert_eq!(cmp, Ordering::Less);

        let cmp = compare_numeric("5", "  10  ", CaseSensitivity::Sensitive);
        assert_eq!(cmp, Ordering::Less);
    }

    #[test]
    fn test_compare_numeric_a_numeric_b_not() {
        let cmp = compare_numeric("5", "apple", CaseSensitivity::Sensitive);
        assert_eq!(cmp, Ordering::Less); // numbers come first
    }

    #[test]
    fn test_compare_numeric_a_not_b_numeric() {
        let cmp = compare_numeric("apple", "5", CaseSensitivity::Sensitive);
        assert_eq!(cmp, Ordering::Greater); // non-numbers come after
    }

    #[test]
    fn test_compare_numeric_both_non_numeric() {
        // Falls back to lexicographic
        let cmp = compare_numeric("apple", "banana", CaseSensitivity::Sensitive);
        assert_eq!(cmp, Ordering::Less);

        let cmp = compare_numeric("banana", "apple", CaseSensitivity::Sensitive);
        assert_eq!(cmp, Ordering::Greater);
    }

    #[test]
    fn test_compare_numeric_mixed_numeric_non_numeric() {
        // Table: [5, apple, 10, banana]
        // After numeric sort: [5, 10, apple, banana]
        let values = vec!["5", "apple", "10", "banana"];
        let mut sorted = values.clone();
        sorted.sort_by(|a, b| compare_numeric(a, b, CaseSensitivity::Sensitive));
        assert_eq!(sorted, vec!["5", "10", "apple", "banana"]);
    }

    // Lexicographic comparisons

    #[test]
    fn test_compare_lexicographic_case_sensitive() {
        let cmp = compare_lexicographic("apple", "banana", CaseSensitivity::Sensitive);
        assert_eq!(cmp, Ordering::Less);

        let cmp = compare_lexicographic("Apple", "apple", CaseSensitivity::Sensitive);
        assert_eq!(cmp, Ordering::Less); // Uppercase comes before lowercase in ASCII
    }

    #[test]
    fn test_compare_lexicographic_case_insensitive() {
        let cmp = compare_lexicographic("Apple", "apple", CaseSensitivity::Insensitive);
        assert_eq!(cmp, Ordering::Equal);

        let cmp = compare_lexicographic("BANANA", "apple", CaseSensitivity::Insensitive);
        assert_eq!(cmp, Ordering::Greater);
    }

    #[test]
    fn test_compare_lexicographic_numbers_as_strings() {
        // When treated as lexicographic, "10" < "5" (string comparison)
        let cmp = compare_lexicographic("10", "5", CaseSensitivity::Sensitive);
        assert_eq!(cmp, Ordering::Less);
    }

    #[test]
    fn test_compare_lexicographic_equal_strings() {
        let cmp = compare_lexicographic("test", "test", CaseSensitivity::Sensitive);
        assert_eq!(cmp, Ordering::Equal);

        let cmp = compare_lexicographic("test", "test", CaseSensitivity::Insensitive);
        assert_eq!(cmp, Ordering::Equal);
    }

    // ========================================================================
    // TASK 4.2: Sort Function Tests
    // ========================================================================

    #[test]
    fn test_sort_table_numeric_ascending() {
        let mut table = Table {
            start_line: 1,
            header: "| Col |".to_string(),
            separator: "|-----|".to_string(),
            rows: vec![
                TableRow {
                    raw: "| 10 |".to_string(),
                    cells: vec!["10".to_string()],
                },
                TableRow {
                    raw: "| 5 |".to_string(),
                    cells: vec!["5".to_string()],
                },
                TableRow {
                    raw: "| 20 |".to_string(),
                    cells: vec!["20".to_string()],
                },
            ],
            column_count: 1,
        };

        let options = SortOptions {
            column: 1,
            order: SortOrder::Asc,
            case: CaseSensitivity::Sensitive,
            sort_type: SortType::Numeric,
        };

        sort_table(&mut table, &options).unwrap();

        assert_eq!(table.rows[0].cells[0], "5");
        assert_eq!(table.rows[1].cells[0], "10");
        assert_eq!(table.rows[2].cells[0], "20");
    }

    #[test]
    fn test_sort_table_numeric_descending() {
        let mut table = Table {
            start_line: 1,
            header: "| Col |".to_string(),
            separator: "|-----|".to_string(),
            rows: vec![
                TableRow {
                    raw: "| 5 |".to_string(),
                    cells: vec!["5".to_string()],
                },
                TableRow {
                    raw: "| 20 |".to_string(),
                    cells: vec!["20".to_string()],
                },
                TableRow {
                    raw: "| 10 |".to_string(),
                    cells: vec!["10".to_string()],
                },
            ],
            column_count: 1,
        };

        let options = SortOptions {
            column: 1,
            order: SortOrder::Desc,
            case: CaseSensitivity::Sensitive,
            sort_type: SortType::Numeric,
        };

        sort_table(&mut table, &options).unwrap();

        assert_eq!(table.rows[0].cells[0], "20");
        assert_eq!(table.rows[1].cells[0], "10");
        assert_eq!(table.rows[2].cells[0], "5");
    }

    #[test]
    fn test_sort_table_lexicographic_case_sensitive() {
        let mut table = Table {
            start_line: 1,
            header: "| Name |".to_string(),
            separator: "|------|".to_string(),
            rows: vec![
                TableRow {
                    raw: "| charlie |".to_string(),
                    cells: vec!["charlie".to_string()],
                },
                TableRow {
                    raw: "| alice |".to_string(),
                    cells: vec!["alice".to_string()],
                },
                TableRow {
                    raw: "| bob |".to_string(),
                    cells: vec!["bob".to_string()],
                },
            ],
            column_count: 1,
        };

        let options = SortOptions {
            column: 1,
            order: SortOrder::Asc,
            case: CaseSensitivity::Sensitive,
            sort_type: SortType::Lexicographic,
        };

        sort_table(&mut table, &options).unwrap();

        assert_eq!(table.rows[0].cells[0], "alice");
        assert_eq!(table.rows[1].cells[0], "bob");
        assert_eq!(table.rows[2].cells[0], "charlie");
    }

    #[test]
    fn test_sort_table_lexicographic_case_insensitive() {
        let mut table = Table {
            start_line: 1,
            header: "| Name |".to_string(),
            separator: "|------|".to_string(),
            rows: vec![
                TableRow {
                    raw: "| Charlie |".to_string(),
                    cells: vec!["Charlie".to_string()],
                },
                TableRow {
                    raw: "| alice |".to_string(),
                    cells: vec!["alice".to_string()],
                },
                TableRow {
                    raw: "| BOB |".to_string(),
                    cells: vec!["BOB".to_string()],
                },
            ],
            column_count: 1,
        };

        let options = SortOptions {
            column: 1,
            order: SortOrder::Asc,
            case: CaseSensitivity::Insensitive,
            sort_type: SortType::Lexicographic,
        };

        sort_table(&mut table, &options).unwrap();

        assert_eq!(table.rows[0].cells[0], "alice");
        assert_eq!(table.rows[1].cells[0], "BOB");
        assert_eq!(table.rows[2].cells[0], "Charlie");
    }

    #[test]
    fn test_sort_table_multi_column() {
        // Sort by column 2 (second column)
        let mut table = Table {
            start_line: 1,
            header: "| A | B |".to_string(),
            separator: "|---|---|".to_string(),
            rows: vec![
                TableRow {
                    raw: "| x | 20 |".to_string(),
                    cells: vec!["x".to_string(), "20".to_string()],
                },
                TableRow {
                    raw: "| y | 10 |".to_string(),
                    cells: vec!["y".to_string(), "10".to_string()],
                },
                TableRow {
                    raw: "| z | 5 |".to_string(),
                    cells: vec!["z".to_string(), "5".to_string()],
                },
            ],
            column_count: 2,
        };

        let options = SortOptions {
            column: 2, // Sort by column 2
            order: SortOrder::Asc,
            case: CaseSensitivity::Sensitive,
            sort_type: SortType::Numeric,
        };

        sort_table(&mut table, &options).unwrap();

        assert_eq!(table.rows[0].cells[0], "z");
        assert_eq!(table.rows[1].cells[0], "y");
        assert_eq!(table.rows[2].cells[0], "x");
    }

    #[test]
    fn test_sort_table_stable_sort_preserves_order() {
        // Create a table where multiple rows have the same sort key
        let mut table = Table {
            start_line: 1,
            header: "| Val | Order |".to_string(),
            separator: "|-----|-------|".to_string(),
            rows: vec![
                TableRow {
                    raw: "| 1 | first |".to_string(),
                    cells: vec!["1".to_string(), "first".to_string()],
                },
                TableRow {
                    raw: "| 1 | second |".to_string(),
                    cells: vec!["1".to_string(), "second".to_string()],
                },
                TableRow {
                    raw: "| 2 | third |".to_string(),
                    cells: vec!["2".to_string(), "third".to_string()],
                },
                TableRow {
                    raw: "| 1 | fourth |".to_string(),
                    cells: vec!["1".to_string(), "fourth".to_string()],
                },
            ],
            column_count: 2,
        };

        let options = SortOptions {
            column: 1,
            order: SortOrder::Asc,
            case: CaseSensitivity::Sensitive,
            sort_type: SortType::Numeric,
        };

        sort_table(&mut table, &options).unwrap();

        // Rows with value "1" should maintain their original relative order
        assert_eq!(table.rows[0].cells[1], "first");
        assert_eq!(table.rows[1].cells[1], "second");
        assert_eq!(table.rows[2].cells[1], "fourth");
        assert_eq!(table.rows[3].cells[1], "third");
    }

    #[test]
    fn test_sort_document_single_table() {
        let mut doc = Document {
            source: None,
            blocks: vec![Block::SortedTable {
                comment_line: "<!-- smt -->".to_string(),
                comment_line_number: 1,
                options: SortOptions {
                    column: 1,
                    order: SortOrder::Asc,
                    case: CaseSensitivity::Sensitive,
                    sort_type: SortType::Numeric,
                },
                table: Table {
                    start_line: 2,
                    header: "| Col |".to_string(),
                    separator: "|-----|".to_string(),
                    rows: vec![
                        TableRow {
                            raw: "| 10 |".to_string(),
                            cells: vec!["10".to_string()],
                        },
                        TableRow {
                            raw: "| 5 |".to_string(),
                            cells: vec!["5".to_string()],
                        },
                    ],
                    column_count: 1,
                },
                blank_lines_after_comment: Vec::new(),
            }],
            line_ending: LineEnding::Lf,
            trailing_newline: false,
        };

        sort_document(&mut doc).unwrap();

        if let Block::SortedTable { table, .. } = &doc.blocks[0] {
            assert_eq!(table.rows[0].cells[0], "5");
            assert_eq!(table.rows[1].cells[0], "10");
        } else {
            panic!("Expected SortedTable block");
        }
    }

    #[test]
    fn test_sort_document_multiple_tables() {
        let mut doc = Document {
            source: None,
            blocks: vec![
                Block::SortedTable {
                    comment_line: "<!-- smt -->".to_string(),
                    comment_line_number: 1,
                    options: SortOptions {
                        column: 1,
                        order: SortOrder::Asc,
                        case: CaseSensitivity::Sensitive,
                        sort_type: SortType::Numeric,
                    },
                    table: Table {
                        start_line: 2,
                        header: "| Col |".to_string(),
                        separator: "|-----|".to_string(),
                        rows: vec![
                            TableRow {
                                raw: "| 10 |".to_string(),
                                cells: vec!["10".to_string()],
                            },
                            TableRow {
                                raw: "| 5 |".to_string(),
                                cells: vec!["5".to_string()],
                            },
                        ],
                        column_count: 1,
                    },
                    blank_lines_after_comment: Vec::new(),
                },
                Block::SortedTable {
                    comment_line: "<!-- smt -->".to_string(),
                    comment_line_number: 10,
                    options: SortOptions {
                        column: 1,
                        order: SortOrder::Asc,
                        case: CaseSensitivity::Sensitive,
                        sort_type: SortType::Lexicographic,
                    },
                    table: Table {
                        start_line: 11,
                        header: "| Name |".to_string(),
                        separator: "|------|".to_string(),
                        rows: vec![
                            TableRow {
                                raw: "| charlie |".to_string(),
                                cells: vec!["charlie".to_string()],
                            },
                            TableRow {
                                raw: "| alice |".to_string(),
                                cells: vec!["alice".to_string()],
                            },
                        ],
                        column_count: 1,
                    },
                    blank_lines_after_comment: Vec::new(),
                },
            ],
            line_ending: LineEnding::Lf,
            trailing_newline: false,
        };

        sort_document(&mut doc).unwrap();

        if let Block::SortedTable { table, .. } = &doc.blocks[0] {
            assert_eq!(table.rows[0].cells[0], "5");
            assert_eq!(table.rows[1].cells[0], "10");
        } else {
            panic!("Expected SortedTable block");
        }

        if let Block::SortedTable { table, .. } = &doc.blocks[1] {
            assert_eq!(table.rows[0].cells[0], "alice");
            assert_eq!(table.rows[1].cells[0], "charlie");
        } else {
            panic!("Expected SortedTable block");
        }
    }

    // ========================================================================
    // TASK 4.3: Check Mode & Edge Case Tests
    // ========================================================================

    #[test]
    fn test_is_table_sorted_already_sorted() {
        let table = Table {
            start_line: 1,
            header: "| Col |".to_string(),
            separator: "|-----|".to_string(),
            rows: vec![
                TableRow {
                    raw: "| 5 |".to_string(),
                    cells: vec!["5".to_string()],
                },
                TableRow {
                    raw: "| 10 |".to_string(),
                    cells: vec!["10".to_string()],
                },
                TableRow {
                    raw: "| 20 |".to_string(),
                    cells: vec!["20".to_string()],
                },
            ],
            column_count: 1,
        };

        let options = SortOptions {
            column: 1,
            order: SortOrder::Asc,
            case: CaseSensitivity::Sensitive,
            sort_type: SortType::Numeric,
        };

        assert!(is_table_sorted(&table, &options));
    }

    #[test]
    fn test_is_table_sorted_unsorted() {
        let table = Table {
            start_line: 1,
            header: "| Col |".to_string(),
            separator: "|-----|".to_string(),
            rows: vec![
                TableRow {
                    raw: "| 10 |".to_string(),
                    cells: vec!["10".to_string()],
                },
                TableRow {
                    raw: "| 5 |".to_string(),
                    cells: vec!["5".to_string()],
                },
                TableRow {
                    raw: "| 20 |".to_string(),
                    cells: vec!["20".to_string()],
                },
            ],
            column_count: 1,
        };

        let options = SortOptions {
            column: 1,
            order: SortOrder::Asc,
            case: CaseSensitivity::Sensitive,
            sort_type: SortType::Numeric,
        };

        assert!(!is_table_sorted(&table, &options));
    }

    #[test]
    fn test_is_table_sorted_descending() {
        let table = Table {
            start_line: 1,
            header: "| Col |".to_string(),
            separator: "|-----|".to_string(),
            rows: vec![
                TableRow {
                    raw: "| 20 |".to_string(),
                    cells: vec!["20".to_string()],
                },
                TableRow {
                    raw: "| 10 |".to_string(),
                    cells: vec!["10".to_string()],
                },
                TableRow {
                    raw: "| 5 |".to_string(),
                    cells: vec!["5".to_string()],
                },
            ],
            column_count: 1,
        };

        let options = SortOptions {
            column: 1,
            order: SortOrder::Desc,
            case: CaseSensitivity::Sensitive,
            sort_type: SortType::Numeric,
        };

        assert!(is_table_sorted(&table, &options));
    }

    #[test]
    fn test_is_table_sorted_single_row() {
        let table = Table {
            start_line: 1,
            header: "| Col |".to_string(),
            separator: "|-----|".to_string(),
            rows: vec![TableRow {
                raw: "| 5 |".to_string(),
                cells: vec!["5".to_string()],
            }],
            column_count: 1,
        };

        let options = SortOptions {
            column: 1,
            order: SortOrder::Asc,
            case: CaseSensitivity::Sensitive,
            sort_type: SortType::Numeric,
        };

        assert!(is_table_sorted(&table, &options));
    }

    #[test]
    fn test_is_table_sorted_empty_table() {
        let table = Table {
            start_line: 1,
            header: "| Col |".to_string(),
            separator: "|-----|".to_string(),
            rows: vec![],
            column_count: 1,
        };

        let options = SortOptions {
            column: 1,
            order: SortOrder::Asc,
            case: CaseSensitivity::Sensitive,
            sort_type: SortType::Numeric,
        };

        assert!(is_table_sorted(&table, &options));
    }

    #[test]
    fn test_is_table_sorted_case_insensitive() {
        let table = Table {
            start_line: 1,
            header: "| Name |".to_string(),
            separator: "|------|".to_string(),
            rows: vec![
                TableRow {
                    raw: "| alice |".to_string(),
                    cells: vec!["alice".to_string()],
                },
                TableRow {
                    raw: "| BOB |".to_string(),
                    cells: vec!["BOB".to_string()],
                },
                TableRow {
                    raw: "| Charlie |".to_string(),
                    cells: vec!["Charlie".to_string()],
                },
            ],
            column_count: 1,
        };

        let options = SortOptions {
            column: 1,
            order: SortOrder::Asc,
            case: CaseSensitivity::Insensitive,
            sort_type: SortType::Lexicographic,
        };

        assert!(is_table_sorted(&table, &options));
    }

    #[test]
    fn test_sort_table_floats() {
        let mut table = Table {
            start_line: 1,
            header: "| Value |".to_string(),
            separator: "|-------|".to_string(),
            rows: vec![
                TableRow {
                    raw: "| 3.14 |".to_string(),
                    cells: vec!["3.14".to_string()],
                },
                TableRow {
                    raw: "| 1.41 |".to_string(),
                    cells: vec!["1.41".to_string()],
                },
                TableRow {
                    raw: "| 2.71 |".to_string(),
                    cells: vec!["2.71".to_string()],
                },
            ],
            column_count: 1,
        };

        let options = SortOptions {
            column: 1,
            order: SortOrder::Asc,
            case: CaseSensitivity::Sensitive,
            sort_type: SortType::Numeric,
        };

        sort_table(&mut table, &options).unwrap();

        assert_eq!(table.rows[0].cells[0], "1.41");
        assert_eq!(table.rows[1].cells[0], "2.71");
        assert_eq!(table.rows[2].cells[0], "3.14");
    }

    #[test]
    fn test_sort_table_negative_numbers() {
        let mut table = Table {
            start_line: 1,
            header: "| Num |".to_string(),
            separator: "|-----|".to_string(),
            rows: vec![
                TableRow {
                    raw: "| -5 |".to_string(),
                    cells: vec!["-5".to_string()],
                },
                TableRow {
                    raw: "| 10 |".to_string(),
                    cells: vec!["10".to_string()],
                },
                TableRow {
                    raw: "| -20 |".to_string(),
                    cells: vec!["-20".to_string()],
                },
            ],
            column_count: 1,
        };

        let options = SortOptions {
            column: 1,
            order: SortOrder::Asc,
            case: CaseSensitivity::Sensitive,
            sort_type: SortType::Numeric,
        };

        sort_table(&mut table, &options).unwrap();

        assert_eq!(table.rows[0].cells[0], "-20");
        assert_eq!(table.rows[1].cells[0], "-5");
        assert_eq!(table.rows[2].cells[0], "10");
    }

    #[test]
    fn test_sort_table_mixed_numeric_non_numeric() {
        let mut table = Table {
            start_line: 1,
            header: "| Val |".to_string(),
            separator: "|-----|".to_string(),
            rows: vec![
                TableRow {
                    raw: "| apple |".to_string(),
                    cells: vec!["apple".to_string()],
                },
                TableRow {
                    raw: "| 5 |".to_string(),
                    cells: vec!["5".to_string()],
                },
                TableRow {
                    raw: "| banana |".to_string(),
                    cells: vec!["banana".to_string()],
                },
                TableRow {
                    raw: "| 10 |".to_string(),
                    cells: vec!["10".to_string()],
                },
            ],
            column_count: 1,
        };

        let options = SortOptions {
            column: 1,
            order: SortOrder::Asc,
            case: CaseSensitivity::Sensitive,
            sort_type: SortType::Numeric,
        };

        sort_table(&mut table, &options).unwrap();

        // Numbers should come first
        assert_eq!(table.rows[0].cells[0], "5");
        assert_eq!(table.rows[1].cells[0], "10");
        // Then non-numeric in stable order
        assert_eq!(table.rows[2].cells[0], "apple");
        assert_eq!(table.rows[3].cells[0], "banana");
    }

    #[test]
    fn test_sort_table_with_whitespace_cells() {
        let mut table = Table {
            start_line: 1,
            header: "| Name |".to_string(),
            separator: "|------|".to_string(),
            rows: vec![
                TableRow {
                    raw: "|charlie|".to_string(),
                    cells: vec!["charlie".to_string()],
                },
                TableRow {
                    raw: "|alice|".to_string(),
                    cells: vec!["alice".to_string()],
                },
                TableRow {
                    raw: "|bob|".to_string(),
                    cells: vec!["bob".to_string()],
                },
            ],
            column_count: 1,
        };

        let options = SortOptions {
            column: 1,
            order: SortOrder::Asc,
            case: CaseSensitivity::Sensitive,
            sort_type: SortType::Lexicographic,
        };

        sort_table(&mut table, &options).unwrap();

        // Should sort in lexicographic order
        assert_eq!(table.rows[0].cells[0], "alice");
        assert_eq!(table.rows[1].cells[0], "bob");
        assert_eq!(table.rows[2].cells[0], "charlie");
    }

    #[test]
    fn test_sort_three_column_table_by_second_column() {
        let mut table = Table {
            start_line: 1,
            header: "| A | B | C |".to_string(),
            separator: "|---|---|---|".to_string(),
            rows: vec![
                TableRow {
                    raw: "| 1 | 30 | x |".to_string(),
                    cells: vec!["1".to_string(), "30".to_string(), "x".to_string()],
                },
                TableRow {
                    raw: "| 2 | 10 | y |".to_string(),
                    cells: vec!["2".to_string(), "10".to_string(), "y".to_string()],
                },
                TableRow {
                    raw: "| 3 | 20 | z |".to_string(),
                    cells: vec!["3".to_string(), "20".to_string(), "z".to_string()],
                },
            ],
            column_count: 3,
        };

        let options = SortOptions {
            column: 2,
            order: SortOrder::Asc,
            case: CaseSensitivity::Sensitive,
            sort_type: SortType::Numeric,
        };

        sort_table(&mut table, &options).unwrap();

        assert_eq!(table.rows[0].cells[0], "2");
        assert_eq!(table.rows[1].cells[0], "3");
        assert_eq!(table.rows[2].cells[0], "1");
    }

    #[test]
    fn test_sort_preserves_raw_column() {
        let mut table = Table {
            start_line: 1,
            header: "| Col |".to_string(),
            separator: "|-----|".to_string(),
            rows: vec![
                TableRow {
                    raw: "| 10 |".to_string(),
                    cells: vec!["10".to_string()],
                },
                TableRow {
                    raw: "| 5 |".to_string(),
                    cells: vec!["5".to_string()],
                },
            ],
            column_count: 1,
        };

        let options = SortOptions {
            column: 1,
            order: SortOrder::Asc,
            case: CaseSensitivity::Sensitive,
            sort_type: SortType::Numeric,
        };

        sort_table(&mut table, &options).unwrap();

        // Raw strings should be preserved (not modified)
        assert_eq!(table.rows[0].raw, "| 5 |");
        assert_eq!(table.rows[1].raw, "| 10 |");
    }

    #[test]
    fn test_compare_numeric_zero() {
        let cmp = compare_numeric("0", "5", CaseSensitivity::Sensitive);
        assert_eq!(cmp, Ordering::Less);

        let cmp = compare_numeric("-5", "0", CaseSensitivity::Sensitive);
        assert_eq!(cmp, Ordering::Less);

        let cmp = compare_numeric("0", "0", CaseSensitivity::Sensitive);
        assert_eq!(cmp, Ordering::Equal);
    }

    #[test]
    fn test_is_table_sorted_does_not_modify_original() {
        let table = Table {
            start_line: 1,
            header: "| Col |".to_string(),
            separator: "|-----|".to_string(),
            rows: vec![
                TableRow {
                    raw: "| 10 |".to_string(),
                    cells: vec!["10".to_string()],
                },
                TableRow {
                    raw: "| 5 |".to_string(),
                    cells: vec!["5".to_string()],
                },
            ],
            column_count: 1,
        };

        let options = SortOptions {
            column: 1,
            order: SortOrder::Asc,
            case: CaseSensitivity::Sensitive,
            sort_type: SortType::Numeric,
        };

        // Call is_table_sorted multiple times to ensure it's immutable
        assert!(!is_table_sorted(&table, &options));
        assert!(!is_table_sorted(&table, &options));

        // Original table should still have original order
        assert_eq!(table.rows[0].cells[0], "10");
        assert_eq!(table.rows[1].cells[0], "5");
    }
}
