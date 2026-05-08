# Technical Design: `smt-core`

## 1. Executive Summary

The `smt-core` change implements a Rust CLI tool that sorts markdown tables opted-in via `<!-- smt -->` HTML comments. The implementation follows a **two-phase pipeline architecture** (parse+sort all inputs, then write results) ensuring atomic file updates: on ANY error, no files are modified.

The baseline architecture is documented in `.agents/ARCHITECTURE.md` with module dependency diagrams, data flow, complete Rust data structures, parser state machine, and atomic write strategy. This design expands that baseline with implementation-level detail.

---

## 2. Module API & Data Structures

See `.agents/ARCHITECTURE.md` for complete Rust data structures. This section summarizes each module's responsibility and public API.

### 2.1 `error.rs` — Error Types (Leaf Module)

**Responsibility**: Define all error types, map to exit codes.

**Key Type**:

```rust
#[derive(Error, Debug)]
pub enum SmtError {
    // CLI errors
    #[error("--write cannot be used with multiple input files")]
    WriteWithMultipleFiles,
    #[error("--append requires --write")]
    AppendWithoutWrite,
    #[error("--in-place cannot be used with stdin")]
    InPlaceWithStdin,
    #[error("no files matched pattern \"{pattern}\"")]
    NoFilesMatched { pattern: String },

    // Parse errors
    #[error("{path}:{line}: smt comment is not followed by a table")]
    CommentWithoutTable { path: SourceLocation, line: usize },
    #[error("{path}:{line}: duplicate smt comment (previous at line {previous_line})")]
    DuplicateComment { path: SourceLocation, line: usize, previous_line: usize },
    #[error("{path}:{line}: unknown option \"{key}\" in smt comment")]
    UnknownOption { path: SourceLocation, line: usize, key: String },
    #[error("{path}:{line}: invalid value \"{value}\" for option \"{key}\" (expected: {expected})")]
    InvalidOptionValue { path: SourceLocation, line: usize, key: String, value: String, expected: String },
    #[error("{path}:{line}: column must be >= 1 in smt comment")]
    ColumnZero { path: SourceLocation, line: usize },
    #[error("{path}:{line}: column must be a positive integer in smt comment")]
    ColumnNotInteger { path: SourceLocation, line: usize },
    #[error("{path}:{line}: column {column} is out of range (table has {actual} columns)")]
    ColumnOutOfRange { path: SourceLocation, line: usize, column: usize, actual: usize },
    #[error("{path}:{line}: malformed table (missing separator row)")]
    MalformedTable { path: SourceLocation, line: usize },

    // I/O errors
    #[error("file not found: {path}")]
    FileNotFound { path: PathBuf },
    #[error("permission denied: {path}")]
    PermissionDenied { path: PathBuf },
    #[error("I/O error: {source}")]
    Io { #[from] source: std::io::Error },
}

pub struct SourceLocation(pub Option<PathBuf>);

impl SmtError {
    pub fn exit_code(&self) -> i32 { 2 } // All errors → exit 2
}
```

**Error Output**: All errors to `stderr` via `eprintln!()` in `main.rs`.

### 2.2 `cli.rs` — CLI Argument Parsing & Validation

**Responsibility**: Parse command-line arguments, validate flag combinations, expand glob patterns, detect stdin/TTY.

**Key Types** (from `.agents/ARCHITECTURE.md`):

```rust
#[derive(Parser, Debug)]
pub struct Args {
    pub inputs: Vec<String>,
    #[arg(short, long, conflicts_with_all = ["write", "check"])]
    pub in_place: bool,
    #[arg(short, long, conflicts_with_all = ["in_place", "check"])]
    pub write: Option<PathBuf>,
    #[arg(long, requires = "write")]
    pub append: bool,
    #[arg(long, conflicts_with_all = ["in_place", "write"])]
    pub check: bool,
    #[arg(long)]
    pub verbose: bool,
}

pub enum InputSource { Stdin, Files(Vec<PathBuf>) }
pub enum OutputTarget { Stdout, InPlace, File { path: PathBuf, append: bool } }
```

**Public Functions**:

- `parse_args() -> Result<(InputSource, OutputTarget, bool, bool), SmtError>` — Parse and validate
- `expand_globs(patterns: Vec<String>) -> Result<Vec<PathBuf>, SmtError>` — Glob expansion
- `detect_input_source(inputs: Vec<String>) -> InputSource` — Stdin vs files

**Implementation Details**:

- Use `clap` with `conflicts_with_all` and `requires` for mutual exclusivity
- Use `glob` crate for pattern expansion
- Detect TTY with `std::io::IsTerminal` (Rust 1.70+)
- All validation errors produce `SmtError` variants

### 2.3 `parser.rs` — Markdown Parsing & Table Extraction

**Responsibility**: Parse markdown, detect `<!-- smt -->` comments, extract and validate tables, build `Document` AST.

**Key Types** (from `.agents/ARCHITECTURE.md`):

```rust
pub struct Document {
    pub source: Option<PathBuf>,
    pub blocks: Vec<Block>,
}

pub enum Block {
    PlainText(Vec<String>),
    SortedTable { comment_line: String, comment_line_number: usize, options: SortOptions, table: Table },
}

pub struct SortOptions {
    pub column: usize,          // 1-based
    pub order: SortOrder,       // Asc | Desc
    pub case: CaseSensitivity,  // Sensitive | Insensitive
    pub sort_type: SortType,    // Numeric | Lexicographic
}

pub enum SortOrder { Asc, Desc }
pub enum CaseSensitivity { Sensitive, Insensitive }
pub enum SortType { Numeric, Lexicographic }

pub struct Table {
    pub start_line: usize,
    pub header: String,
    pub separator: String,
    pub rows: Vec<TableRow>,
    pub column_count: usize,
}

pub struct TableRow {
    pub raw: String,        // Preserve for lossless output
    pub cells: Vec<String>, // Trimmed for sorting
}
```

**Public Functions**:

- `parse(content: &str, source: Option<PathBuf>) -> Result<Document, SmtError>` — Parse markdown into Document
- `parse_sort_options(comment_text: &str, comment_line: usize, source: Option<PathBuf>) -> Result<SortOptions, SmtError>` — Parse options

**Implementation Details**:

- **State machine** (see `.agents/ARCHITECTURE.md` “Parser State Machine”): Normal → ExpectTable → ExpectSep → ReadingRows → Normal
- **Line classification**:
  - SMT comment: `^\s*<!--\s+smt(\s+.*)?\s*-->\s*$`
  - Table row: `^\s*\|.*\|\s*$`
  - Separator: is a table row AND all cells match `^:?-+:?$`
  - Plain text: everything else
- **Comment parsing** (11 steps, no regex):
  1. Strip whitespace
  2. Strip `<!--` prefix → Strip `-->` suffix
  3. Strip `smt` prefix
  4. If empty, return `SortOptions::default()`
  5. Split by whitespace
  6. For each token: split on `=`, validate key, parse value
  7. Return `SortOptions` with parsed + default values
- **Table validation**: Column count matches all rows, separator is valid
- **Error detection**: Comment without table, duplicate comments, column out of range
- **Lossless preservation**: Store `raw` lines (not formatted) in `TableRow`

### 2.4 `sorter.rs` — Sorting Logic

**Responsibility**: Sort table rows according to `SortOptions`, support numeric/lexicographic comparison, case sensitivity, sort direction.

**Key Types**:

```rust
pub struct CheckResult {
    pub source: Option<PathBuf>,
    pub comment_line: usize,
    pub table_start_line: usize,
    pub is_sorted: bool,
}
```

**Public Functions**:

- `sort_document(doc: &mut Document) -> Result<(), SmtError>` — In-place sort all tables in document
- `is_table_sorted(table: &Table, options: &SortOptions) -> bool` — Check if table is already sorted (for `--check`)

**Implementation Details**:

- **Comparator design**:
  - Extract sort key from column: `table.rows[i].cells[column]`
  - For **numeric**: try `parse::<f64>()`. Non-numeric → sort after all numeric (stable relative order among non-numerics)
  - For **lexicographic**: use `.to_lowercase()` if `case=insensitive`, else as-is. Standard `std::cmp::Ord` on `String`
  - Apply sort direction: reverse comparator for `Desc`
- **Stability guarantee**: Use `slice::sort_by` (stable), NEVER `sort_unstable_by`
- **`is_table_sorted()`**: Clone rows, sort clone, compare to original

### 2.5 `writer.rs` — Output Writing

**Responsibility**: Render `Document` back to markdown string, write to stdout/file/in-place.

**Public Functions**:

- `render(doc: &Document) -> String` — Convert Document back to markdown
- `write(doc: &Document, target: &OutputTarget, source: Option<PathBuf>) -> Result<(), SmtError>` — Write to target

**Implementation Details**:

- **Rendering**:
  ```rust
  fn render(document: &Document) -> String {
      let mut output = String::new();
      for block in &document.blocks {
          match block {
              Block::PlainText(lines) => {
                  for line in lines {
                      output.push_str(line);
                      output.push('\n');
                  }
              }
              Block::SortedTable { comment_line, table, .. } => {
                  output.push_str(comment_line);
                  output.push('\n');
                  output.push_str(&table.header);
                  output.push('\n');
                  output.push_str(&table.separator);
                  output.push('\n');
                  for row in &table.rows {
                      output.push_str(&row.raw);
                      output.push('\n');
                  }
              }
          }
      }
      output
  }
  ```
- **Write targets**:
  - `Stdout`: Print to stdout via `println!()`
  - `File { path, append }`: Write to file via `std::fs::write()` or `std::fs::OpenOptions::append()`
  - `InPlace`: Use `tempfile::NamedTempFile`, write, fsync, `persist()` for atomic rename
- **Error handling**: Wrap I/O errors in `SmtError::Io`

---

## 3. Data Flow & Pipeline

See `.agents/ARCHITECTURE.md` for the full diagram. Implementation follows strict two-phase design:

```
PHASE 1: Parse & Sort (All files)
  └─ Read all inputs (files or stdin)
  └─ For each input:
     ├─ parser::parse() → Document
     ├─ sorter::sort_document(&mut doc) → in-place sort
     └─ Collect result or error
  └─ If ANY error: print to stderr, exit 2, NO files written

PHASE 2: Write (Only if Phase 1 succeeds)
  └─ For each (document, output_target):
     └─ If --check:
        ├─ For each SortedTable: compare sorted vs original
        ├─ Collect unsorted locations
        ├─ If any unsorted: print to stdout (if --verbose), exit 1
     └─ Else:
        └─ writer::write(document, target) → stdout/file/in-place
  └─ Exit 0 on success
```

**Critical Rule**: Once Phase 1 completes successfully for ALL inputs, Phase 2 writes are guaranteed. No partial writes on error.

---

## 4. Parser State Machine

See `.agents/ARCHITECTURE.md` for the full FSM diagram.

**Implementation**:

```rust
enum ParserState {
    Normal,      // Reading plain text
    ExpectTable, // Saw `<!-- smt -->`, expect header next
    ExpectSep,   // Saw header, expect separator next
    ReadingRows, // Reading data rows
}
```

**Line processing loop**:

```rust
for (line_num, line) in lines.iter().enumerate() {
    match state {
        Normal => {
            if is_smt_comment(line) {
                // Store comment, transition to ExpectTable
            } else if is_table_row(line) {
                // Add to PlainText block
            } else {
                // Add to PlainText block
            }
        }
        ExpectTable => {
            if is_table_row(line) {
                // Extract header, transition to ExpectSep
            } else {
                // Error: comment without table
            }
        }
        ExpectSep => {
            if is_separator_row(line) {
                // Extract separator, transition to ReadingRows
            } else {
                // Error: malformed table
            }
        }
        ReadingRows => {
            if is_table_row(line) {
                // Add to rows
            } else {
                // Finalize SortedTable block, emit, transition to Normal
            }
        }
    }
}
```

---

## 5. Sorting Algorithm

**Comparator pattern**:

```rust
fn sort_rows(rows: &mut Vec<TableRow>, options: &SortOptions, table: &Table) {
    let col_idx = options.column - 1; // Convert 1-based to 0-based

    rows.sort_by(|a, b| {
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
}

fn compare_numeric(a: &str, b: &str, case: CaseSensitivity) -> Ordering {
    let a_num = a.trim().parse::<f64>().ok();
    let b_num = b.trim().parse::<f64>().ok();

    match (a_num, b_num) {
        (Some(an), Some(bn)) => an.partial_cmp(&bn).unwrap_or(Ordering::Equal),
        (Some(_), None) => Ordering::Less,     // Numbers before non-numbers
        (None, Some(_)) => Ordering::Greater,
        (None, None) => compare_lexicographic(a, b, case),
    }
}

fn compare_lexicographic(a: &str, b: &str, case: CaseSensitivity) -> Ordering {
    let a_str = match case {
        CaseSensitivity::Insensitive => a.to_lowercase(),
        CaseSensitivity::Sensitive => a.to_string(),
    };
    let b_str = match case {
        CaseSensitivity::Insensitive => b.to_lowercase(),
        CaseSensitivity::Sensitive => b.to_string(),
    };

    a_str.cmp(&b_str)
}
```

---

## 6. Atomic Write Strategy for `-i` Mode

See `.agents/ARCHITECTURE.md` Section “Atomic Write Strategy for `-i` Mode”.

**Implementation** (using `tempfile` crate):

```rust
use tempfile::NamedTempFile;
use std::fs;

fn write_in_place(path: &Path, content: &str) -> Result<(), SmtError> {
    // Create temp file in same directory (ensures same filesystem)
    let parent = path.parent().unwrap_or_else(|| Path::new("."));
    let mut temp = NamedTempFile::new_in(parent)
        .map_err(|e| SmtError::Io { source: e })?;

    // Write content
    use std::io::Write;
    temp.write_all(content.as_bytes())
        .map_err(|e| SmtError::Io { source: e })?;

    // Sync to disk
    temp.flush()
        .map_err(|e| SmtError::Io { source: e })?;

    // Atomic rename (POSIX atomic on same filesystem)
    temp.persist(path)
        .map_err(|e| SmtError::Io { source: e.error })?;

    Ok(())
}
```

**Atomicity guarantee (project rule)**: On ANY error, **no files are modified** (including failures during the write phase). This is stricter than “per-file atomic writes”.

**Design implication**: For multi-file `--in-place`, Phase 2 must be transactional across the whole set:

1. **Prepare**: for every target file, write the new content to a temp file in the same directory (same filesystem) and fsync it.
2. **Commit**: attempt to swap all originals to backups, then persist all temp files to their final names.
3. **Rollback on any failure**: if any rename/persist fails, restore any files already moved/replaced from their backups so the workspace returns to its original state.

This requires creating backups (e.g. `path.md.smt.bak`) during the commit step so rollback is possible if a later rename fails.

---

## 7. Error Handling & Exit Codes

**Error mapping** (see `.agents/PLAN.md` “Exit Codes” and `.agents/ARCHITECTURE.md` “Error Type Hierarchy”):

- All `SmtError` variants → exit code **2** (user error)
- `--check` unsorted → exit code **1** (not an error, just a check result) — handled in `main.rs` before calling `SmtError::exit_code()`
- Success → exit code **0**

**Output routing**:

- Errors → `stderr` via `eprintln!("{}", error)`
- Normal sorted output → `stdout` via `println!()`
- `--check --verbose` unsorted table reports → `stdout` (informational, not errors)

---

## 8. Testing Strategy

See `.agents/PLAN.md` Section “Testing Strategy” for the full testing plan.

**Unit Tests** (in each src file):

```rust
#[cfg(test)]
mod tests {
    use super::*;

    // Parser tests
    #[test]
    fn test_parse_default_options() { }

    #[test]
    fn test_parse_with_column_and_order() { }

    #[test]
    fn test_parse_unknown_option_error() { }

    // Sorter tests
    #[test]
    fn test_numeric_sort() { }

    #[test]
    fn test_stable_sort() { }

    // Writer tests
    #[test]
    fn test_render_preserves_formatting() { }
}
```

**Integration Tests** (`tests/integration_test.rs`):

- Use `assert_cmd` crate to run compiled binary
- Use `predicates` crate for output assertions
- Use `tempfile` for temporary test files
- Test fixtures: `.md` files in `tests/fixtures/input/`, `.expected.md` for expected outputs

Example:

```rust
#[test]
fn test_sort_single_file_to_stdout() {
    Command::cargo_bin("smt")
        .unwrap()
        .arg("tests/fixtures/input/simple.md")
        .assert().success()
        .stdout(predicate::str::contains("| Alice"));
}

#[test]
fn test_check_unsorted() {
    Command::cargo_bin("smt")
        .unwrap()
        .arg("--check")
        .arg("tests/fixtures/unsorted.md")
        .assert()
        .code(1);
}
```

---

## 9. File Layout & Build

**Cargo.toml**:

```toml
[package]
name = "smt"
version = "0.1.0"
edition = "2021"

[dependencies]
clap = { version = "4.4", features = ["derive"] }
thiserror = "2.0"
anyhow = "1.0"
glob = "0.3"
tempfile = "3.8"

[dev-dependencies]
assert_cmd = "2.0"
predicates = "3.0"
```

**Directory structure**:

```
src/
├── main.rs      # Entry point, orchestrates pipeline
├── cli.rs       # CLI parsing & validation
├── parser.rs    # Markdown parsing
├── sorter.rs    # Sort logic
├── writer.rs    # Output handling
└── error.rs     # Error types

tests/
├── integration_test.rs
└── fixtures/
    ├── input/
    │   ├── simple.md
    │   ├── numeric_sort.md
    │   └── ...
    ├── expected/
    │   ├── simple.expected.md
    │   └── ...
    └── unsorted/
        └── unsorted.md
```

**Build command**: `cargo build --release` → `target/release/smt`

---

## 10. Implementation Phases (High-Level)

### Phase 1: Core Module Infrastructure

- Set up `Cargo.toml` with dependencies
- Implement `error.rs` with `SmtError` enum
- Implement `cli.rs` with `Args`, input/output validation, glob expansion
- Write unit tests for each module

### Phase 2: Parser & Sorter

- Implement `parser.rs` state machine, comment parsing, table extraction
- Implement `sorter.rs` comparators (numeric, lexicographic, case sensitivity)
- Write comprehensive unit tests for edge cases

### Phase 3: Writer & Integration

- Implement `writer.rs` rendering and output targets (stdout, file, in-place)
- Implement `main.rs` orchestration: Phase 1 parse+sort, Phase 2 write, exit code handling
- Test two-phase atomicity

### Phase 4: Full Testing & Polish

- Write all integration tests with fixtures
- Test `--check` mode, `--verbose`, mutual exclusivity constraints
- Test error paths (permission denied, file not found, malformed tables, etc.)
- Performance validation (single binary, <10ms startup)

---

## Key Design Decisions

1. **Two-phase pipeline**: Guarantees atomicity — parse/sort all before writing any
2. **Stable sort only**: `slice::sort_by`, deterministic output
3. **Hand-rolled parsing**: No regex, just `str` methods — simpler, faster
4. **Lossless round-trip**: Store raw lines, reconstruct exactly
5. **Atomic file writes**: `tempfile` + persist for `-i` mode
6. **Clear error messages**: Include line numbers and context for debugging

---

## Next Step

Run `/sdd-tasks smt-core` to produce the detailed task breakdown for implementation.
