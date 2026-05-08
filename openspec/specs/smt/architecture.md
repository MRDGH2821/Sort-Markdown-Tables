# `smt` — Architecture Document

## Module Dependency Diagram

```
                    ┌──────────┐
                    │  main.rs │
                    └────┬─────┘
                         │
            ┌────────────┼────────────┐
            │            │            │
            ▼            ▼            ▼
       ┌────────┐  ┌──────────┐  ┌──────────┐
       │  cli   │  │  parser  │  │  writer   │
       └────┬───┘  └────┬─────┘  └────┬─────┘
            │            │             │
            │            ▼             │
            │       ┌──────────┐      │
            │       │  sorter  │      │
            │       └────┬─────┘      │
            │            │            │
            ▼            ▼            ▼
       ┌──────────────────────────────────┐
       │             error                │
       └──────────────────────────────────┘
```

### Dependency Rules

- `main.rs` depends on ALL modules — it's the orchestrator
- `cli` depends on `error` (for validation errors)
- `parser` depends on `error` (for parse errors) and uses `sorter` types (`SortOptions`)
- `sorter` depends on `error` (for sort errors like column out of range) and `parser` types (`Table`, `SortOptions`)
- `writer` depends on `error` (for I/O errors) and `parser` types (`Document`)
- `error` depends on nothing — it's the leaf module
- NO circular dependencies

---

## Data Flow

### Full Pipeline

```
┌─────────────────────────────────────────────────────────────────────┐
│                          PHASE 1: Parse & Sort                      │
│                                                                     │
│  ┌───────────┐    ┌──────────────┐    ┌──────────────┐             │
│  │ cli::run() │───▶│ Read file(s) │───▶│ parser::     │             │
│  │            │    │ or stdin     │    │ parse()      │             │
│  └───────────┘    └──────────────┘    └──────┬───────┘             │
│                                              │                      │
│                                     Document { blocks }             │
│                                              │                      │
│                                              ▼                      │
│                                     ┌──────────────┐               │
│                                     │ sorter::      │               │
│                                     │ sort_document()│              │
│                                     └──────┬───────┘               │
│                                            │                        │
│                              Sorted Document (or Error)             │
│                                            │                        │
│                         ┌──────────────────┼──────────────┐        │
│                         │                  │              │        │
│                    (collect all)      (any error?)   (--check?)    │
│                         │                  │              │        │
│                         ▼                  ▼              ▼        │
│                    Vec<Result>        ABORT ALL     compare()      │
│                                      exit 2        exit 0 or 1    │
└─────────────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────────────┐
│                        PHASE 2: Write (only if Phase 1 succeeded)   │
│                                                                     │
│  Vec<(Document, OutputTarget)>                                      │
│           │                                                         │
│           ▼                                                         │
│  ┌──────────────┐     ┌──────────┐     ┌──────────┐               │
│  │ writer::     │────▶│  stdout   │     │  file    │               │
│  │ write_all()  │────▶│           │ or  │ (atomic) │               │
│  └──────────────┘     └──────────┘     └──────────┘               │
│                                                                     │
│           exit 0                                                    │
└─────────────────────────────────────────────────────────────────────┘
```

### Step-by-Step

1. **CLI Parsing** (`cli`): Parse arguments, expand globs, validate flag combinations
2. **Read Input**: Read file contents into memory (all files, before any processing)
3. **Parse** (`parser`): For each input, tokenize into `Document` — a sequence of `Block`s
4. **Sort** (`sorter`): For each `SortedTable` block in each `Document`, sort data rows
5. **Collect Results**: If any file produced an error, abort entirely (exit 2)
6. **Check Mode Branch**: If `--check`, compare sorted vs original for each table. Exit 0 or 1.
7. **Write** (`writer`): Write all documents to their targets (stdout, file, or in-place)

---

## Key Data Structures

### `cli.rs`

```rust
use clap::Parser;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(name = "smt", version, about = "Sort Markdown Tables")]
pub struct Args {
    /// Input files or glob patterns
    #[arg()]
    pub inputs: Vec<String>,

    /// Sort tables in-place
    #[arg(short, long, conflicts_with_all = ["write", "check"])]
    pub in_place: bool,

    /// Write output to a specific file
    #[arg(short, long, conflicts_with_all = ["in_place", "check"])]
    pub write: Option<PathBuf>,

    /// Append to output file instead of overwriting (requires --write)
    #[arg(long, requires = "write")]
    pub append: bool,

    /// Check if tables are sorted (exit 1 if not)
    #[arg(long, conflicts_with_all = ["in_place", "write"])]
    pub check: bool,

    /// Print unsorted table locations in check mode
    #[arg(long)]
    pub verbose: bool,
}

/// Resolved input source after CLI validation
pub enum InputSource {
    Stdin,
    Files(Vec<PathBuf>),
}

/// Resolved output target after CLI validation
pub enum OutputTarget {
    Stdout,
    InPlace,                        // write back to source file
    File { path: PathBuf, append: bool },
}
```

### `parser.rs`

```rust
/// A parsed markdown document, preserving structure for lossless round-tripping
pub struct Document {
    /// Source file path (None for stdin)
    pub source: Option<PathBuf>,
    /// Ordered sequence of blocks that make up the document
    pub blocks: Vec<Block>,
}

/// A block within the document
pub enum Block {
    /// Lines of text that are NOT part of an smt-controlled table.
    /// Stored verbatim to enable lossless output.
    PlainText(Vec<String>),

    /// An smt comment + the table it controls
    SortedTable {
        /// The raw `<!-- smt ... -->` comment line (preserved for output)
        comment_line: String,
        /// Line number of the comment in the source file (1-based, for errors)
        comment_line_number: usize,
        /// Parsed sort options from the comment
        options: SortOptions,
        /// The table structure
        table: Table,
    },
}

/// Parsed options from an `<!-- smt ... -->` comment
#[derive(Debug, Clone, PartialEq)]
pub struct SortOptions {
    pub column: usize,          // 1-based, default 1
    pub order: SortOrder,       // default Asc
    pub case: CaseSensitivity,  // default Sensitive
    pub sort_type: SortType,    // default Numeric
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
    /// Line number where the table starts (header row, 1-based)
    pub start_line: usize,
    /// The header row, e.g., `| Name | Age |`
    pub header: String,
    /// The separator row, e.g., `|------|-----|`
    pub separator: String,
    /// Data rows — these are what get sorted
    pub rows: Vec<TableRow>,
    /// Number of columns (derived from header)
    pub column_count: usize,
}

/// A single data row in a table
#[derive(Debug, Clone)]
pub struct TableRow {
    /// The raw line as it appears in the source (for lossless output)
    pub raw: String,
    /// Parsed cell contents (trimmed, for sorting)
    pub cells: Vec<String>,
}
```

### `sorter.rs`

```rust
/// Result of checking whether a table is already sorted
pub struct CheckResult {
    pub source: Option<PathBuf>,
    pub comment_line: usize,
    pub table_start_line: usize,
    pub is_sorted: bool,
}
```

The sorter module operates on `&mut Table` in-place, reordering `table.rows` according to `SortOptions`. It also exposes a `is_sorted()` function for `--check` mode that returns `CheckResult` without mutating.

### `error.rs`

```rust
use std::path::PathBuf;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum SmtError {
    // --- CLI errors ---
    #[error("--write cannot be used with multiple input files")]
    WriteWithMultipleFiles,

    #[error("--append requires --write")]
    AppendWithoutWrite,

    #[error("--in-place cannot be used with stdin")]
    InPlaceWithStdin,

    #[error("no files matched pattern \"{pattern}\"")]
    NoFilesMatched { pattern: String },

    // --- Parse errors ---
    #[error("{path}:{line}: smt comment is not followed by a table")]
    CommentWithoutTable {
        path: SourceLocation,
        line: usize,
    },

    #[error("{path}:{line}: duplicate smt comment (previous at line {previous_line})")]
    DuplicateComment {
        path: SourceLocation,
        line: usize,
        previous_line: usize,
    },

    #[error("{path}:{line}: unknown option \"{key}\" in smt comment")]
    UnknownOption {
        path: SourceLocation,
        line: usize,
        key: String,
    },

    #[error("{path}:{line}: invalid value \"{value}\" for option \"{key}\" (expected: {expected})")]
    InvalidOptionValue {
        path: SourceLocation,
        line: usize,
        key: String,
        value: String,
        expected: String,
    },

    #[error("{path}:{line}: column must be >= 1 in smt comment")]
    ColumnZero {
        path: SourceLocation,
        line: usize,
    },

    #[error("{path}:{line}: column must be a positive integer in smt comment")]
    ColumnNotInteger {
        path: SourceLocation,
        line: usize,
    },

    #[error("{path}:{line}: column {column} is out of range (table has {actual} columns)")]
    ColumnOutOfRange {
        path: SourceLocation,
        line: usize,
        column: usize,
        actual: usize,
    },

    #[error("{path}:{line}: malformed table (missing separator row)")]
    MalformedTable {
        path: SourceLocation,
        line: usize,
    },

    // --- I/O errors ---
    #[error("file not found: {path}")]
    FileNotFound { path: PathBuf },

    #[error("permission denied: {path}")]
    PermissionDenied { path: PathBuf },

    #[error("I/O error: {source}")]
    Io {
        #[from]
        source: std::io::Error,
    },
}

/// Displayable source location (file path or "<stdin>")
#[derive(Debug, Clone)]
pub struct SourceLocation(pub Option<PathBuf>);

impl std::fmt::Display for SourceLocation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.0 {
            Some(path) => write!(f, "{}", path.display()),
            None => write!(f, "<stdin>"),
        }
    }
}

impl SmtError {
    /// Map error to process exit code
    pub fn exit_code(&self) -> i32 {
        2 // All errors are user errors (exit 2)
          // Exit code 1 is reserved for --check mode (handled in main)
    }
}
```

---

## Error Type Hierarchy

```
SmtError (enum)
├── CLI Validation Errors
│   ├── WriteWithMultipleFiles
│   ├── AppendWithoutWrite
│   ├── InPlaceWithStdin
│   └── NoFilesMatched { pattern }
│
├── Parse Errors
│   ├── CommentWithoutTable { path, line }
│   ├── DuplicateComment { path, line, previous_line }
│   ├── UnknownOption { path, line, key }
│   ├── InvalidOptionValue { path, line, key, value, expected }
│   ├── ColumnZero { path, line }
│   ├── ColumnNotInteger { path, line }
│   ├── ColumnOutOfRange { path, line, column, actual }
│   └── MalformedTable { path, line }
│
└── I/O Errors
    ├── FileNotFound { path }
    ├── PermissionDenied { path }
    └── Io { source: std::io::Error }
```

All variants map to exit code **2**. Exit code **1** is NOT an error — it's a check result, handled by `main.rs` directly (not via `SmtError`).

---

## Parser State Machine

The parser operates line-by-line with the following states:

```
                     ┌──────────┐
                     │  Normal  │◀──────────────────────────────┐
                     └────┬─────┘                               │
                          │                                     │
               ┌──────────┴──────────┐                          │
               │ Is this line an     │                          │
               │ <!-- smt --> comment│                          │
               │ ?                   │                          │
               └──────┬──────┬───────┘                          │
                  Yes │      │ No                               │
                      │      │                                  │
                      │      ▼                                  │
                      │  ┌──────────────┐                       │
                      │  │ Is this line │                       │
                      │  │ a table row? │                       │
                      │  └──┬───────┬───┘                       │
                      │  Yes│       │No                         │
                      │     │       │                           │
                      │     ▼       ▼                           │
                      │  (standalone  (add to current           │
                      │   table —     PlainText block)          │
                      │   no smt                                │
                      │   comment —                             │
                      │   pass through                          │
                      │   untouched)                            │
                      │                                         │
                      ▼                                         │
               ┌──────────────┐                                 │
               │ ExpectTable  │                                 │
               │ (have comment│                                 │
               │  need header)│                                 │
               └──────┬───────┘                                 │
                      │                                         │
               ┌──────┴──────┐                                  │
               │ Next line a │                                  │
               │ table header│                                  │
               │ ?           │                                  │
               └──┬──────┬───┘                                  │
              Yes │      │ No → ERROR: comment without table    │
                  │                                             │
                  ▼                                             │
           ┌──────────────┐                                     │
           │ ExpectSep    │                                     │
           │ (have header,│                                     │
           │  need sep)   │                                     │
           └──────┬───────┘                                     │
                  │                                             │
           ┌──────┴──────┐                                      │
           │ Next line a │                                      │
           │ separator?  │                                      │
           └──┬──────┬───┘                                      │
          Yes │      │ No → ERROR: malformed table              │
              │                                                 │
              ▼                                                 │
       ┌──────────────┐                                         │
       │ ReadingRows  │                                         │
       │ (collecting  │                                         │
       │  data rows)  │                                         │
       └──────┬───────┘                                         │
              │                                                 │
       ┌──────┴──────┐                                          │
       │ Next line a │                                          │
       │ table row?  │                                          │
       └──┬──────┬───┘                                          │
      Yes │      │ No                                           │
          │      │                                              │
          ▼      ▼                                              │
       (add to   (finalize SortedTable block,                   │
        rows)     emit it, return to Normal) ───────────────────┘
```

### Line Classification

A line is classified as:

| Classification | Rule                                                                        |
| -------------- | --------------------------------------------------------------------------- |
| SMT comment    | Matches `^\s*<!--\s+smt(\s+.*)?\s*-->\s*$`                                  |
| Table row      | Matches `^\s*\|.*\|\s*$` (starts and ends with pipe after trimming)         |
| Separator row  | Is a table row AND all cells match `^:?-+:?$` (dashes with optional colons) |
| Plain text     | Everything else                                                             |

---

## Atomic Write Strategy for `-i` Mode

```
For each file in input:
    1. Read original content
    2. Parse into Document
    3. Sort tables in Document
    4. Render Document back to string
    5. Store (path, new_content) in Vec

If any step above failed for ANY file:
    → Print error to stderr
    → Exit 2
    → NO files are modified

Once all files processed successfully:
    For each (path, new_content):
        1. Write to temp file in same directory (same filesystem)
        2. fsync temp file
        3. Rename temp file over original (atomic on POSIX)
        4. If rename fails, attempt cleanup of temp file
```

Using the same directory for temp files ensures the rename is atomic (same filesystem). The `tempfile` crate's `NamedTempFile::persist()` handles this correctly.

---

## `--check` Mode Flow

```
For each file in input:
    1. Parse into Document (same as normal)
    2. For each SortedTable block:
        a. Clone the data rows
        b. Sort the clone
        c. Compare clone to original
        d. If different → record (file, comment_line, table_start_line)

If any table was unsorted:
    if --verbose:
        Print each unsorted table location to stdout:
            "file.md:7: table is not sorted (comment at line 6)"
    Exit 1

If all tables were sorted:
    Exit 0
```

---

## Comment Parsing Detail

Given the line `<!-- smt column=2 order=desc -->`:

```
1. Strip leading/trailing whitespace
2. Strip "<!--" prefix → " smt column=2 order=desc -->"
3. Strip "-->" suffix → " smt column=2 order=desc "
4. Trim → "smt column=2 order=desc"
5. Strip "smt" prefix → " column=2 order=desc"
6. If step 5 fails (doesn't start with "smt") → not an smt comment
7. Trim → "column=2 order=desc"
8. If empty → return SortOptions::default()
9. Split by whitespace → ["column=2", "order=desc"]
10. For each token:
    a. Split on '=' → (key, value)
    b. If no '=' found → error: malformed option
    c. Match key against known keys
    d. If unknown → error: unknown option
    e. Parse value according to key's type
    f. If invalid value → error: invalid value
11. Return SortOptions with parsed values (defaults for unspecified keys)
```

No regex needed. Pure `str` methods.

---

## Rendering Back to Markdown

The `Document` → `String` rendering is straightforward because we preserved everything:

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

Note: Line ending handling (LF vs CRLF) needs care. The parser should detect the line ending style of the input and the renderer should match it. For v1, supporting LF only is acceptable since markdown files are overwhelmingly LF.

**Important**: The last line of the file may or may not have a trailing newline. The parser must track this to avoid adding or removing a trailing newline.
