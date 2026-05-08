# Task Breakdown: `smt-core` — Core Implementation

## Overview

This document lists all implementation tasks in hierarchical order, grouped by phase. Each task is an atomic unit completable in 1–3 hours with clear success criteria and explicit dependencies.

**Total Tasks**: 21  
**Estimated Effort**: 35–45 man-hours  
**Critical Path**: Phase 1 → Phase 2 → Phase 3 → Phase 4 → Phase 5 → Phase 6 → Phase 7

---

## Phase 1: Project Setup & Dependencies

Initialize Cargo project, set up module structure, add all external dependencies (clap, thiserror, anyhow, glob, tempfile, assert_cmd, predicates), and create test directory hierarchy.

### 1.1 Initialize Cargo.toml with Core Dependencies

**Description**: Create or update `Cargo.toml` with all required crates and dev-dependencies.

**Checklist**:

- [ ] Create `Cargo.toml` with `[package]` section (name: `smt`, version: `0.1.0`, edition: `2021`)
- [ ] Add production dependencies: `clap` (4.4+, derive feature), `thiserror` (2.0+), `anyhow` (1.0+), `glob` (0.3+), `tempfile` (3.8+)
- [ ] Add dev-dependencies: `assert_cmd` (2.0+), `predicates` (3.0+)
- [ ] Verify `cargo check` compiles without errors (may have unused warnings, that's OK)

**Dependencies**: None  
**Success Criteria**:

- `Cargo.toml` exists with all required crates pinned to minimum versions
- `cargo check` completes without compilation errors
- No missing transitive dependencies

**Time Estimate**: 20 minutes

---

### 1.2 Create Module Structure & Declarations

**Description**: Set up `src/` directory with all module files and proper module declarations.

**Checklist**:

- [ ] Create `src/main.rs` with module declarations (no implementation, just `mod error; mod cli; mod parser; mod sorter; mod writer;`)
- [ ] Create empty files: `src/error.rs`, `src/cli.rs`, `src/parser.rs`, `src/sorter.rs`, `src/writer.rs`
- [ ] Ensure `cargo build` compiles successfully (will have warnings about unused code, that's OK)
- [ ] Create `tests/` directory

**Dependencies**: 1.1  
**Success Criteria**:

- All 5 source modules are declared in `main.rs`
- `cargo build` succeeds with no errors
- All modules are importable (e.g., `cargo check` passes)

**Time Estimate**: 15 minutes

---

### 1.3 Create Test Directory Structure & Fixtures Template

**Description**: Set up test fixtures directory with template files for integration tests.

**Checklist**:

- [ ] Create `tests/fixtures/input/` directory
- [ ] Create `tests/fixtures/expected/` directory
- [ ] Create `tests/fixtures/unsorted/` directory
- [ ] Create `tests/integration_test.rs` stub (empty test function for now)
- [ ] Create at least 2 template `.md` files: `simple.md` (basic table with smt comment), `unsorted.md` (table that needs sorting)
- [ ] Create corresponding `.expected.md` files with expected sorted output

**Dependencies**: 1.2  
**Success Criteria**:

- Directory structure exists and `cargo test --test integration_test` runs (even if empty)
- At least 2 pairs of (input.md, input.expected.md) exist in `tests/fixtures/`
- Fixtures can be read by integration test runner (no I/O errors)

**Time Estimate**: 30 minutes

---

## Phase 2: Core Error & CLI Modules

Implement error handling and command-line argument parsing with validation.

### 2.1 Implement `error.rs` — SmtError Enum & Exit Code Mapping

**Description**: Create the complete `SmtError` enum with all error variants and exit code mapping, as specified in `design.md` Section 2.1.

**Checklist**:

- [ ] Define `#[derive(Error, Debug)]` enum `SmtError` with variants:
  - CLI errors: `WriteWithMultipleFiles`, `AppendWithoutWrite`, `InPlaceWithStdin`, `NoFilesMatched`
  - Parse errors: `CommentWithoutTable`, `DuplicateComment`, `UnknownOption`, `InvalidOptionValue`, `ColumnZero`, `ColumnNotInteger`, `ColumnOutOfRange`, `MalformedTable`
  - I/O errors: `FileNotFound`, `PermissionDenied`, `Io` (wrapped from `std::io::Error`)
- [ ] Implement `impl SmtError` with `pub fn exit_code(&self) -> i32` returning 2 for all errors
- [ ] Add `#[error(...)]` attributes to each variant with descriptive messages (including placeholders like `{path}:{line}:`)
- [ ] Define `SourceLocation` type (wrapper around `Option<PathBuf>`)
- [ ] Write unit tests for each error variant message format and exit code

**Dependencies**: 1.2  
**Success Criteria**:

- All 14+ error variants defined with clear messages
- `exit_code()` returns 2 for all variants
- Unit tests cover all variants with sample paths/line numbers
- `cargo build` compiles without errors

**Time Estimate**: 1 hour

---

### 2.2 Implement `cli.rs` — Argument Parsing & Validation

**Description**: Create `Args` struct with Clap derive macros, input/output detection, validation logic, and glob expansion.

**Checklist**:

- [ ] Define `#[derive(Parser, Debug)]` struct `Args` with:
  - `inputs: Vec<String>` (positional arguments)
  - `in_place: bool` (short: `-i`, long: `--in-place`, conflicts with `write` and `check`)
  - `write: Option<PathBuf>` (short: `-w`, long: `--write`, conflicts with `in_place` and `check`)
  - `append: bool` (long: `--append`, requires `write`)
  - `check: bool` (long: `--check`, conflicts with `in_place` and `write`)
  - `verbose: bool` (long: `--verbose`)
- [ ] Define `enum InputSource { Stdin, Files(Vec<PathBuf>) }`
- [ ] Define `enum OutputTarget { Stdout, InPlace, File { path: PathBuf, append: bool } }`
- [ ] Implement `parse_args() -> Result<(InputSource, OutputTarget, bool, bool), SmtError>`:
  - Detect TTY with `std::io::IsTerminal` (Rust 1.70+)
  - If no inputs and TTY stdin: return help message and exit (TODO: implement in `main.rs`)
  - If no inputs and non-TTY stdin: set input source to `Stdin`
  - If inputs provided: expand globs, validate combinations
- [ ] Implement `expand_globs(patterns: Vec<String>) -> Result<Vec<PathBuf>, SmtError>`:
  - Use `glob` crate to expand each pattern
  - Error if zero files matched: return `SmtError::NoFilesMatched`
- [ ] Implement validation logic:
  - `-i` and `-w` are mutually exclusive
  - `--check` and (`-i` or `-w`) are mutually exclusive
  - `--append` requires `-w` (Clap `requires` attribute)
  - `-w` with multiple files: error
  - `-i` with stdin: error
- [ ] Write unit tests for all flag combinations, glob expansion, input detection

**Dependencies**: 2.1  
**Success Criteria**:

- All flag parsing works correctly
- Glob expansion returns file list or error
- Mutual exclusivity constraints enforced
- Input/output detection works (TTY detection, stdin vs files)
- 15+ unit tests covering all validation rules

**Time Estimate**: 1.5 hours

---

### 2.3 Write Unit Tests for Error & CLI Modules

**Description**: Comprehensive unit tests for `error.rs` and `cli.rs`.

**Checklist**:

- [ ] Test each `SmtError` variant produces correct message format
- [ ] Test `exit_code()` returns 2 for all variants
- [ ] Test `Args` parsing with all flag combinations
- [ ] Test mutual exclusivity (e.g., `-i -w` should error early)
- [ ] Test glob expansion with wildcards, empty matches, special characters
- [ ] Test TTY detection (mock `IsTerminal` if needed, or use conditional compilation)
- [ ] Test input source detection: stdin vs files
- [ ] Test output target mapping: stdout, in-place, file, append
- [ ] Run `cargo test --lib error cli` and ensure 100% pass

**Dependencies**: 2.2  
**Success Criteria**:

- 20+ unit tests in `error.rs` and `cli.rs`
- All tests pass with `cargo test`
- Edge cases covered: empty patterns, special chars in filenames, conflicting flags

**Time Estimate**: 45 minutes

---

## Phase 3: Parser Module

Implement markdown parsing with state machine, comment detection, table extraction, and validation.

### 3.1 Implement Parser Data Structures

**Description**: Define all data types used by the parser module.

**Checklist**:

- [x] Define `enum Block { PlainText(Vec<String>), SortedTable { comment_line: String, comment_line_number: usize, options: SortOptions, table: Table } }`
- [x] Define `struct Document { source: Option<PathBuf>, blocks: Vec<Block> }`
- [x] Define `enum SortOrder { Asc, Desc }`
- [x] Define `enum CaseSensitivity { Sensitive, Insensitive }`
- [x] Define `enum SortType { Numeric, Lexicographic }`
- [x] Define `struct SortOptions { column: usize, order: SortOrder, case: CaseSensitivity, sort_type: SortType }`
- [x] Implement `SortOptions::default()` returning `SortOptions { column: 1, order: SortOrder::Asc, case: CaseSensitivity::Sensitive, sort_type: SortType::Numeric }`
- [x] Define `struct Table { start_line: usize, header: String, separator: String, rows: Vec<TableRow>, column_count: usize }`
- [x] Define `struct TableRow { raw: String, cells: Vec<String> }`

**Dependencies**: 1.2  
**Success Criteria**:

- All types defined and derive correct traits (Debug, Clone where needed)
- `Default` trait implemented for `SortOptions`
- No compilation errors

**Time Estimate**: 30 minutes

---

### 3.2 Implement Comment Parsing Logic (11-Step Algorithm)

**Description**: Hand-rolled comment parsing without regex, extracting key=value pairs and validating options.

**Checklist**:

- [x] Implement `parse_comment(comment_text: &str, line_num: usize, source: Option<PathBuf>) -> Result<SortOptions, SmtError>`:
  - Step 1: Strip leading/trailing whitespace
  - Step 2: Strip `<!--` prefix and `-->` suffix
  - Step 3: Strip `smt` keyword
  - Step 4: If empty, return `SortOptions::default()`
  - Step 5: Split remainder by whitespace into tokens
  - Step 6–9: For each token, parse `key=value`:
    - Split on `=` → key and value parts
    - Validate key in {`column`, `order`, `case`, `type`}
    - Validate value: `column` must be positive integer, `order` in {`asc`, `desc`}, `case` in {`sensitive`, `insensitive`}, `type` in {`numeric`, `lexicographic`}
    - Return `SmtError::UnknownOption` or `SmtError::InvalidOptionValue` on failure
  - Step 10: Collect parsed options
  - Step 11: Return `SortOptions` with parsed + defaults
- [x] Write 15+ unit tests covering:
  - Default options (empty comment)
  - Single option, multiple options
  - Unknown keys, invalid values
  - Column edge cases (0, negative, non-integer, out-of-range)
  - Case sensitivity options
  - Sort type options
  - Whitespace handling

**Dependencies**: 3.1  
**Success Criteria**:

- `parse_comment()` correctly parses all valid option combinations
- All error cases produce correct `SmtError` variant
- 15+ unit tests, all pass

**Time Estimate**: 1.5 hours

---

### 3.3 Implement Parser State Machine & Line Classification

**Description**: Implement the core parser loop with state machine, line classification, and table extraction.

**Checklist**:

- [x] Implement line classification helpers:
  - `is_smt_comment(line: &str) -> bool` — matches `^\s*<!--\s+smt(\s+.*)?\s*-->\s*$`
  - `is_table_row(line: &str) -> bool` — matches `^\s*\|.*\|\s*$`
  - `is_separator_row(line: &str) -> bool` — checks if all cells match `^:?-+:?$`
- [x] Implement parser state machine with `enum ParserState { Normal, ExpectTable, ExpectSep, ReadingRows }`
- [x] Implement `parse(content: &str, source: Option<PathBuf>) -> Result<Document, SmtError>`:
  - Split content by lines
  - Iterate with state machine, classifying each line
  - Build `Document` with `Block` instances (PlainText or SortedTable)
  - Handle state transitions per `design.md` Section 4
  - On error (comment without table, malformed table, etc.), return `SmtError`
- [x] Validate tables:
  - Column count matches all rows
  - Separator row is valid
  - Comment immediately precedes table (no blank lines)
- [x] Write 20+ unit tests covering:
  - Simple valid tables
  - Unmarked tables (no smt comment)
  - Multiple tables in one document
  - Comment without table, duplicate comments
  - Malformed tables (missing separator, mismatched columns)
  - Lossless preservation of raw lines

**Dependencies**: 3.2  
**Success Criteria**:

- `parse()` returns `Document` with correct block structure
- All error cases handled with descriptive messages
- 20+ unit tests, all pass
- Raw lines preserved for output fidelity

**Time Estimate**: 2 hours

---

### 3.4 Write Comprehensive Parser Unit Tests

**Description**: Additional parser tests for edge cases and integration with comment parsing.

**Checklist**:

- [x] Test parsing valid markdown with multiple tables
- [x] Test duplicate smt comments on same table (error)
- [x] Test smt comment with no table following (error)
- [x] Test column count validation (mismatched columns error)
- [x] Test separator row validation (multiple variants of valid separators: `---`, `:---`, `---:`, `:---:`)
- [x] Test whitespace preservation (raw lines, indentation)
- [x] Test plain text blocks between tables
- [x] Test edge case: table at end of file (no trailing newline)
- [x] Run `cargo test --lib parser` and ensure all pass

**Dependencies**: 3.3  
**Success Criteria**:

- 25+ parser unit tests
- All edge cases covered
- All tests pass

**Time Estimate**: 1 hour

---

## Phase 4: Sorter Module

Implement sorting logic with numeric/lexicographic comparators, stability guarantee, and check mode.

### 4.1 Implement Sorter Data Structures & Comparators

**Description**: Implement `CheckResult` type and comparator functions.

**Checklist**:

- [ ] Define `struct CheckResult { source: Option<PathBuf>, comment_line: usize, table_start_line: usize, is_sorted: bool }`
- [ ] Implement `compare_numeric(a: &str, b: &str, case: CaseSensitivity) -> Ordering`:
  - Try parse both as `f64`
  - If both numeric: compare as floats (handle `NaN` via `partial_cmp`)
  - If `a` numeric, `b` non-numeric: `Ordering::Less` (numbers first)
  - If `a` non-numeric, `b` numeric: `Ordering::Greater`
  - If both non-numeric: fallback to lexicographic comparison
- [ ] Implement `compare_lexicographic(a: &str, b: &str, case: CaseSensitivity) -> Ordering`:
  - If case-insensitive: convert both to lowercase, compare
  - If case-sensitive: compare as-is
  - Use `std::cmp::Ord` on `String`
- [ ] Write unit tests for all comparison scenarios

**Dependencies**: 3.1  
**Success Criteria**:

- Comparators correctly handle numeric, lexicographic, case sensitivity
- Mixed numeric/non-numeric cells sort correctly
- 12+ unit tests, all pass

**Time Estimate**: 1 hour

---

### 4.2 Implement Sort Function & Stability Guarantee

**Description**: Implement in-place table sorting using stable `sort_by`.

**Checklist**:

- [ ] Implement `sort_table(table: &mut Table, options: &SortOptions) -> Result<(), SmtError>`:
  - Validate column is in range (already done by parser, but double-check)
  - Convert 1-based column index to 0-based
  - Call `rows.sort_by()` with custom comparator
  - Use pattern from `design.md` Section 5: extract cell, apply comparator, handle direction
  - NEVER use `sort_unstable_by`
- [ ] Implement `sort_document(doc: &mut Document) -> Result<(), SmtError>`:
  - Iterate through blocks
  - For each `SortedTable` block, call `sort_table()`
  - Return error if any table fails
- [ ] Implement `is_table_sorted(table: &Table, options: &SortOptions) -> bool`:
  - Clone rows, sort clone, compare to original
  - Return true if no changes
- [ ] Write unit tests for:
  - Single table sort
  - Multiple tables in document
  - Stability preservation (equal elements retain order)
  - Direction handling (asc vs desc)
  - Numeric and lexicographic sorts

**Dependencies**: 4.1  
**Success Criteria**:

- `sort_table()` and `sort_document()` work correctly
- `is_table_sorted()` correctly detects sorted state
- Stability verified: equal rows maintain original order
- 15+ unit tests, all pass

**Time Estimate**: 1.5 hours

---

### 4.3 Write Comprehensive Sorter Tests

**Description**: Additional sorter tests for edge cases and check mode.

**Checklist**:

- [ ] Test numeric sort with integers, floats, negatives, mixed
- [ ] Test lexicographic sort (case-sensitive and insensitive)
- [ ] Test sort direction (ascending, descending)
- [ ] Test stability with duplicate sort keys
- [ ] Test edge cases: single row, zero rows, table already sorted
- [ ] Test `is_table_sorted()` for all scenarios
- [ ] Run `cargo test --lib sorter` and ensure all pass

**Dependencies**: 4.2  
**Success Criteria**:

- 20+ sorter unit tests
- All edge cases covered
- All tests pass

**Time Estimate**: 1 hour

---

## Phase 5: Writer Module & Main Integration

Implement output rendering and orchestration of the two-phase pipeline.

### 5.1 Implement `writer.rs` — Document Rendering & Output Targets

**Description**: Render `Document` back to markdown and write to various output targets.

**Checklist**:

- [ ] Implement `render(doc: &Document) -> String`:
  - Iterate through blocks
  - For `PlainText`: concatenate lines with newlines
  - For `SortedTable`: output comment line, header, separator, then rows (using `row.raw`)
  - Return complete markdown string
- [ ] Implement `write(doc: &Document, target: &OutputTarget, source: Option<PathBuf>) -> Result<(), SmtError>`:
  - Match on target:
    - `Stdout`: render and print to stdout via `println!()`
    - `File { path, append: false }`: render and write to file (overwrite)
    - `File { path, append: true }`: render and append to file
    - `InPlace`: render, write atomically (see 5.2)
  - All I/O errors wrapped in `SmtError::Io`
- [ ] Write unit tests for rendering and output

**Dependencies**: 3.1, 4.2  
**Success Criteria**:

- `render()` produces valid markdown with sorted tables
- `write()` handles all output targets correctly
- Atomic write via tempfile for `-i` mode
- 10+ unit tests, all pass

**Time Estimate**: 1.5 hours

---

### 5.2 Implement Atomic In-Place Write Strategy

**Description**: Use `tempfile` crate to ensure atomicity for `-i` mode.

**Checklist**:

- [ ] Implement atomic write helper in `writer.rs`:
  - Create `NamedTempFile` in same directory as target
  - Write rendered content to temp file
  - Call `flush()` to sync to disk
  - Call `persist()` to atomically rename
  - Return error if any step fails
- [ ] Validate atomicity:
  - On Phase 1 error, temp files are never created
  - On Phase 2 error during write, original file is never corrupted
- [ ] Write unit tests for atomic write behavior

**Dependencies**: 5.1  
**Success Criteria**:

- Atomic write successfully renames temp file
- Original file preserved on error
- 5+ unit tests for atomicity

**Time Estimate**: 45 minutes

---

### 5.3 Implement `main.rs` — Two-Phase Pipeline Orchestration

**Description**: Entry point that orchestrates Phase 1 (parse & sort) and Phase 2 (write), with proper exit code handling.

**Checklist**:

- [ ] Implement `main()` function:
  - Parse args via `cli::parse_args()`
  - Determine input source and output target
  - **Phase 1: Parse & Sort**:
    - Read all inputs (files or stdin)
    - For each input: `parser::parse()` → `sorter::sort_document()`
    - Collect results or error
    - If any error: print to stderr, exit 2
  - **Phase 2: Write** (only if Phase 1 succeeds):
    - If `--check` mode:
      - For each document: check if sorted via `sorter::is_table_sorted()` for each table
      - Collect unsorted locations
      - If any unsorted: print locations (if `--verbose`) to stdout, exit 1
      - Else: exit 0
    - Else:
      - For each document: `writer::write()` to target
      - Handle output target (stdout, file, in-place) correctly
      - Exit 0
- [ ] Handle special cases:
  - No inputs and TTY stdin: print help, exit 0
  - No inputs and non-TTY stdin: read from stdin, process normally
  - `--check` mode: don't write, just verify
  - `--verbose` with `--check`: print unsorted table locations
- [ ] Error handling:
  - All errors printed to stderr via `eprintln!()`
  - Correct exit codes: 0 (success), 1 (check failed), 2 (user error)
- [ ] Write unit/integration tests for main pipeline

**Dependencies**: 2.2, 3.3, 4.2, 5.1  
**Success Criteria**:

- Two-phase pipeline works correctly
- Phase 1 errors prevent Phase 2 writes (atomicity)
- Exit codes correct: 0, 1, 2 for all scenarios
- Help message on no-input + TTY
- Stdin reading on no-input + non-TTY
- 10+ integration tests for main pipeline

**Time Estimate**: 2 hours

---

### 5.4 Write Integration Tests for Two-Phase Pipeline

**Description**: Test the complete pipeline with realistic scenarios.

**Checklist**:

- [ ] Test single file to stdout: correct output, exit 0
- [ ] Test single file with `-i`: file modified, exit 0
- [ ] Test multiple files with `-i`: all modified or all unchanged on error
- [ ] Test `-w` with single file: output.md created, exit 0
- [ ] Test `-w --append`: content appended, exit 0
- [ ] Test `--check` with sorted file: exit 0, no output
- [ ] Test `--check` with unsorted file: exit 1, no output
- [ ] Test `--check --verbose` with unsorted: prints location, exit 1
- [ ] Test flag conflicts: `-i -w`, `--check -i`, etc. → exit 2
- [ ] Test glob patterns: expand and process all files
- [ ] Test glob with zero matches: error, exit 2
- [ ] Test atomicity: error in one file prevents all writes
- [ ] Run `cargo test --test integration_test` and ensure all pass

**Dependencies**: 5.3  
**Success Criteria**:

- 15+ integration tests
- All scenarios from spec covered
- Atomicity verified with multi-file test
- All tests pass

**Time Estimate**: 2 hours

---

## Phase 6: Integration Testing & Test Fixtures

Create comprehensive test fixtures and integration test suite.

### 6.1 Create Test Fixtures (Input & Expected Output Pairs)

**Description**: Build a comprehensive set of test fixtures in `tests/fixtures/`.

**Checklist**:

- [ ] **simple.md + simple.expected.md**: Basic table with default sort options
- [ ] **numeric_sort.md + numeric_sort.expected.md**: Numeric sort with integers
- [ ] **float_sort.md + float_sort.expected.md**: Float numeric sort
- [ ] **mixed_numeric.md + mixed_numeric.expected.md**: Mixed numeric and non-numeric (numeric first)
- [ ] **lexicographic.md + lexicographic.expected.md**: Lexicographic sort (case-sensitive)
- [ ] **case_insensitive.md + case_insensitive.expected.md**: Lexicographic case-insensitive
- [ ] **descending.md + descending.expected.md**: Descending order
- [ ] **multiple_tables.md + multiple_tables.expected.md**: Multiple marked tables
- [ ] **unmarked_tables.md + unmarked_tables.expected.md**: Mix of marked and unmarked (unmarked unchanged)
- [ ] **empty_table.md**: Table with no data rows (no-op)
- [ ] **single_row.md**: Table with one data row (no-op)
- [ ] **preserve_whitespace.md + preserve_whitespace.expected.md**: Complex formatting preserved
- [ ] **already_sorted.md**: Table already in correct order (no-op)
- [ ] **unsorted.md**: Table that needs sorting (for `--check` test)

**Checklist (Per Fixture)**:

- [ ] Input file has valid markdown table with `<!-- smt -->` comment (where applicable)
- [ ] Expected file shows correct sorted result
- [ ] Comment syntax is correct per spec (e.g., `<!-- smt column=2 order=desc -->`)
- [ ] All fixtures use realistic data (names, numbers, etc.)

**Dependencies**: 1.3  
**Success Criteria**:

- 14+ pairs of fixture files created
- All fixtures compile to valid markdown
- Each fixture covers a distinct scenario from spec
- Fixtures ready for integration tests

**Time Estimate**: 1.5 hours

---

### 6.2 Write Comprehensive Integration Tests

**Description**: Full integration test suite using `assert_cmd` and `predicates`.

**Checklist**:

- [ ] Test: sort simple file to stdout → output matches expected
- [ ] Test: sort file with `-i` → file modified correctly
- [ ] Test: sort to new file with `-w` → new file created with correct output
- [ ] Test: `--append` → content appended to file
- [ ] Test: `--check` on sorted file → exit 0
- [ ] Test: `--check` on unsorted file → exit 1
- [ ] Test: `--check --verbose` on unsorted → prints table location, exit 1
- [ ] Test: multiple files with `-i` all succeed → all modified
- [ ] Test: multiple files with `-i`, one fails → none modified (atomicity)
- [ ] Test: glob pattern matching multiple files → all processed
- [ ] Test: glob with zero matches → error, exit 2
- [ ] Test: `-i -w` together → error, exit 2
- [ ] Test: `--check -i` → error, exit 2
- [ ] Test: `--check -w` → error, exit 2
- [ ] Test: `-w` with multiple files → error, exit 2
- [ ] Test: `--append` without `-w` → error, exit 2
- [ ] Test: stdin + `-i` → error, exit 2
- [ ] Test: stdin + `-w` → allowed, works
- [ ] Test: no args + TTY → prints help, exit 0
- [ ] Test: no args + non-TTY stdin → reads from stdin, works
- [ ] Test: unsorted numeric table → sorts correctly
- [ ] Test: case-insensitive sort → correct order
- [ ] Test: stable sort → equal rows preserve order
- [ ] Test: comment with invalid option → error message, exit 2
- [ ] Test: comment with column out of range → error message, exit 2
- [ ] Run `cargo test --test integration_test` and ensure all pass

**Dependencies**: 6.1  
**Success Criteria**:

- 26+ integration tests
- All spec scenarios covered
- All tests pass
- Atomicity verified

**Time Estimate**: 3 hours

---

### 6.3 Add Additional Edge Case Tests

**Description**: Tests for boundary conditions and error recovery.

**Checklist**:

- [ ] Test: file with no marked tables → output unchanged, exit 0
- [ ] Test: table with single data row → no reordering
- [ ] Test: table with zero data rows → no-op
- [ ] Test: large table (100+ rows) → sorts in <1s
- [ ] Test: malformed table (missing separator) → error, exit 2
- [ ] Test: duplicate smt comments → error, exit 2
- [ ] Test: smt comment with no table following → error, exit 2
- [ ] Test: column count mismatch → error, exit 2
- [ ] Test: smt comment with trailing text (invalid) → error, exit 2
- [ ] Test: file with CRLF line endings → preserved in output
- [ ] Test: very long lines → handled correctly
- [ ] Test: unicode characters in table → sorted correctly
- [ ] Test: permission denied on file read → error, exit 2
- [ ] Test: permission denied on file write → error, exit 2

**Dependencies**: 6.2  
**Success Criteria**:

- 14+ additional edge case tests
- All pass
- Error messages are descriptive

**Time Estimate**: 1.5 hours

---

## Phase 7: Polish & Verification

Final checks, performance validation, and pre-commit hook testing.

### 7.1 Code Review & Cleanup

**Description**: Review all code for style, safety, and correctness.

**Checklist**:

- [ ] Run `cargo fmt` to format all code
- [ ] Run `cargo clippy` and fix all warnings
- [ ] Review all error messages for clarity and consistency
- [ ] Check that NO `unsafe` code exists (except where required by dependencies)
- [ ] Verify all module documentation is present (doc comments for public functions)
- [ ] Ensure consistent error handling: all I/O errors wrapped in `SmtError`
- [ ] Check that all string literals are correct (no typos, formatting)

**Dependencies**: 6.3  
**Success Criteria**:

- `cargo fmt` produces no changes (already formatted)
- `cargo clippy` produces no warnings
- All error messages are clear and actionable
- Zero unsafe code blocks
- Public API documented

**Time Estimate**: 1 hour

---

### 7.2 Performance Validation

**Description**: Verify binary performance meets non-functional requirements.

**Checklist**:

- [ ] Build release binary: `cargo build --release`
- [ ] Measure startup time: `time ./target/release/smt --help` → <10ms
- [ ] Measure parsing + sorting time with large fixture (100+ tables) → <1s
- [ ] Binary size acceptable: <10MB (typical for Rust CLI with no external deps)
- [ ] Memory usage reasonable: no unbounded allocations
- [ ] Test on both debug and release builds

**Dependencies**: 7.1  
**Success Criteria**:

- Startup time <10ms
- Large file processing <1s
- Binary size <10MB

**Time Estimate**: 30 minutes

---

### 7.3 Final Integration Test Run & Verification

**Description**: Full test suite pass with code coverage.

**Checklist**:

- [ ] Run `cargo test --all` (unit + integration tests) → all pass
- [ ] Run `cargo test --release` → all pass (catch any debug-only issues)
- [ ] Verify test output shows 50+ tests
- [ ] Check that all main code paths are tested
- [ ] Run `cargo build --release` → compiles without warnings

**Dependencies**: 7.2  
**Success Criteria**:

- All 50+ tests pass
- No compiler warnings
- Code coverage includes all main paths

**Time Estimate**: 45 minutes

---

### 7.4 Pre-Commit Hook Testing

**Description**: Validate tool works correctly in pre-commit hook scenarios.

**Checklist**:

- [ ] Test: `smt -i "docs/**/*.md"` with glob pattern → all files processed
- [ ] Test: `smt --check "docs/**/*.md"` → exit 0 if sorted, exit 1 if not
- [ ] Test: `smt --check --verbose "docs/**/*.md"` → prints unsorted locations
- [ ] Test: integration with actual pre-commit hook (if applicable)
- [ ] Verify exit codes match spec: 0 (pass), 1 (check fail), 2 (error)

**Dependencies**: 7.3  
**Success Criteria**:

- Pre-commit hook scenarios work correctly
- Exit codes correct for CI/CD pipelines
- Performance acceptable for pre-commit use

**Time Estimate**: 30 minutes

---

### 7.5 Documentation & Version Bump

**Description**: Update project documentation and prepare for release.

**Checklist**:

- [ ] Ensure `README.md` exists (if required by project)
- [ ] Verify all public API has documentation
- [ ] Update `Cargo.toml` version to `0.1.0` (initial release)
- [ ] Check `openspec/specs/smt/plan.md` and `openspec/specs/smt/architecture.md` against implementation
- [ ] Create or update `CHANGELOG.md` (if required)
- [ ] Verify `cargo publish --dry-run` works (if publishing to crates.io)

**Dependencies**: 7.4  
**Success Criteria**:

- Documentation complete and accurate
- Version bumped appropriately
- Ready for release

**Time Estimate**: 30 minutes

---

## Summary

| Phase     | Tasks   | Description           | Est. Hours |
| --------- | ------- | --------------------- | ---------- |
| 1         | 1.1–1.3 | Project Setup         | 1          |
| 2         | 2.1–2.3 | Error & CLI           | 3          |
| 3         | 3.1–3.4 | Parser                | 6.5        |
| 4         | 4.1–4.3 | Sorter                | 3.5        |
| 5         | 5.1–5.4 | Writer & Main         | 6.5        |
| 6         | 6.1–6.3 | Integration Testing   | 6          |
| 7         | 7.1–7.5 | Polish & Verification | 3          |
| **Total** | **21**  |                       | **30–35**  |

---

## Recommended Implementation Batches

### Batch 1: Foundation (Phase 1 & 2) — 4 hours

- 1.1, 1.2, 1.3, 2.1, 2.2, 2.3

**Next**: `/sdd-apply smt-core --batch 1`

### Batch 2: Parser (Phase 3) — 6.5 hours

- 3.1, 3.2, 3.3, 3.4

**Next**: `/sdd-apply smt-core --batch 2`

### Batch 3: Sorter & Writer (Phase 4 & 5) — 10 hours

- 4.1, 4.2, 4.3, 5.1, 5.2, 5.3, 5.4

**Next**: `/sdd-apply smt-core --batch 3`

### Batch 4: Integration (Phase 6) — 6 hours

- 6.1, 6.2, 6.3

**Next**: `/sdd-apply smt-core --batch 4`

### Batch 5: Polish (Phase 7) — 3.5 hours

- 7.1, 7.2, 7.3, 7.4, 7.5

**Next**: `/sdd-verify smt-core`

---

## Critical Dependencies

```
1.1 → 1.2 → 1.3
        ↓
      2.1 → 2.2 → 2.3
              ↓       ↓
            3.1 → 3.2 → 3.3 → 3.4
                    ↓
                  4.1 → 4.2 → 4.3 → 5.1 → 5.2 → 5.3 → 5.4 → 6.1 → 6.2 → 6.3 → 7.1 → 7.2 → 7.3 → 7.4 → 7.5
```

---

## Success Criteria (Phase 7 Verification)

- [ ] All 21 tasks marked complete
- [ ] All 50+ unit tests pass
- [ ] All 26+ integration tests pass
- [ ] All 14+ edge case tests pass
- [ ] No compiler warnings
- [ ] Binary compiles to <10MB
- [ ] Startup time <10ms
- [ ] Large file processing <1s
- [ ] Two-phase atomicity verified
- [ ] Exit codes correct: 0 (success), 1 (check fail), 2 (error)
- [ ] All error messages descriptive
- [ ] Zero unsafe code blocks
- [ ] Markdown fidelity preserved (lossless round-trip)
- [ ] Pre-commit hook scenarios work
- [ ] Ready for release

---

## Next Step

Run `/sdd-apply smt-core --batch 1` to begin Phase 1 & 2 implementation.
