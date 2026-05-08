# Specification: smt-core — Core Implementation of Sort Markdown Tables

## 1. Overview

`smt` is a Rust CLI tool that parses markdown files, identifies tables preceded by `<!-- smt -->` HTML comments, sorts them according to specified options (column, sort direction, case sensitivity, sort type), and outputs results to stdout, files, or in-place with **guaranteed atomicity**. The tool is designed for CI pipelines and pre-commit hooks with explicit exit codes and zero runtime dependencies.

This spec is migrated from (and should fully replace) the canonical specs in `openspec/specs/smt/plan.md` and `openspec/specs/smt/architecture.md`.

---

## 2. Functional Requirements

### 2.1 Comment Syntax & Parsing

**Requirement**: Tables MUST be opted-in for sorting by placing an `<!-- smt -->` HTML comment on the line **immediately** preceding the table header (with no blank lines between). The comment MUST match the pattern `<!-- smt [key=value ...] -->` with optional whitespace around the comment content. Unknown option keys or invalid values MUST produce an error with exit code 2. All options not specified in the comment MUST default to: `column=1, order=asc, case=sensitive, type=numeric`.

#### Scenarios

**Given** a markdown file with a valid `<!-- smt -->` comment immediately above a table:

```markdown
<!-- smt -->

| Name | Age |
| ---- | --- |
| Bob  | 30  |
```

**When** parsed, **Then** use default sort options (column=1, order=asc, case=sensitive, type=numeric) to sort the table.

---

**Given** a comment with mixed valid options: `<!-- smt column=3 order=desc case=insensitive -->`:
**When** parsed, **Then** extract `column=3`, `order=desc`, `case=insensitive`, and apply default for `type=numeric`.

---

**Given** a comment with unknown option key: `<!-- smt colum=2 -->` (typo: `colum` instead of `column`):
**When** parsed, **Then** emit error `"unknown option \"colum\" in smt comment at line X"` and exit with code 2.

---

**Given** a comment with invalid value: `<!-- smt order=ascending -->`:
**When** parsed, **Then** emit error `"invalid value \"ascending\" for option \"order\" (expected: asc, desc)"` and exit with code 2.

---

**Given** a comment with `column=0`:
**When** parsed, **Then** emit error `"column must be >= 1 in smt comment"` and exit with code 2.

---

**Given** a comment with non-integer column: `<!-- smt column=abc -->`:
**When** parsed, **Then** emit error `"column must be a positive integer in smt comment"` and exit with code 2.

---

**Given** a comment with leading/trailing spaces: `<!--  smt  column=2  -->`:
**When** parsed, **Then** treat as valid and extract `column=2`.

---

**Given** an `<!-- smt -->` comment **not** immediately followed by a table header (e.g., a blank line or plain text in between):
**When** parsed, **Then** emit error `"smt comment at line X is not followed by a table"` and exit with code 2.

---

**Given** two consecutive `<!-- smt -->` comments before the same table:
**When** parsed, **Then** emit error `"duplicate smt comment at line Y (previous at line X)"` and exit with code 2.

---

### 2.2 Sorting Behavior

**Requirement**: Only data rows (rows after the header and separator) SHALL be sorted. Header and separator rows MUST remain pinned at the top of each table. The sort MUST be stable (equal rows preserve their original relative order). Numeric sort MUST parse cell content as `f64`; non-numeric cells MUST sort after numeric cells. Lexicographic sort MUST respect case sensitivity per the `case` option. The sort direction (`asc` or `desc`) MUST be honored.

#### Scenarios

**Given** a table with a header, separator, and 5 data rows with `<!-- smt column=2 -->`:

```markdown
| ID  | Name  |
| --- | ----- |
| 3   | Bob   |
| 1   | Alice |
| 2   | Carol |
| 1   | David |
| 4   | Eve   |
```

**When** sorted by column 2 (Name) in ascending order, **Then** only the data rows reorder (header and separator stay at top):

```markdown
| ID  | Name  |
| --- | ----- |
| 1   | Alice |
| 3   | Bob   |
| 2   | Carol |
| 1   | David |
| 4   | Eve   |
```

---

**Given** two data rows with equal values in the sort column: `| 1 | Alice |` and `| 2 | Alice |` in that order:
**When** sorted by column 2 (Name) with `type=lexicographic`, **Then** their relative order is preserved (row with ID 1 stays before row with ID 2).

---

**Given** a table with mixed numeric and non-numeric content in the sort column:

```markdown
<!-- smt type=numeric -->

| Rank | Score |
| ---- | ----- |
| 2    | apple |
| 1    | 95    |
| 3    | 87    |
| 4    | zoo   |
```

**When** sorted numerically by column 2, **Then** numeric cells (95, 87) sort first in numeric order, followed by non-numeric cells (apple, zoo) in their original relative order:

```markdown
| Rank | Score |
| ---- | ----- |
| 3    | 87    |
| 1    | 95    |
| 2    | apple |
| 4    | zoo   |
```

---

**Given** a table with `<!-- smt type=lexicographic case=insensitive -->`:

```markdown
| Word   |
| ------ |
| Zebra  |
| apple  |
| Banana |
```

**When** sorted lexicographically (case-insensitive), **Then** sort by lowercase equivalents but preserve original casing in output:

```markdown
| Word   |
| ------ |
| apple  |
| Banana |
| Zebra  |
```

---

**Given** a table with `<!-- smt order=desc -->` and numeric content:

```markdown
| Value |
| ----- |
| 10    |
| 5     |
| 20    |
```

**When** sorted numerically in descending order, **Then** rows are sorted by column 1 from highest to lowest:

```markdown
| Value |
| ----- |
| 20    |
| 10    |
| 5     |
```

---

**Given** a column that doesn't exist in the table (e.g., `<!-- smt column=5 -->` on a 3-column table):
**When** parsed, **Then** emit error `"column 5 is out of range (table has 3 columns) at line X"` and exit with code 2.

---

**Given** a table with a single data row:
**When** sorted, **Then** no reordering occurs (no-op, table unchanged).

---

**Given** a table with zero data rows (only header and separator):
**When** sorted, **Then** no-op, table unchanged.

---

### 2.3 CLI Interface & Flags

**Requirement**: The CLI SHALL accept zero or more positional arguments (file paths or glob patterns), with flags for in-place sorting (`-i`), writing to a specific file (`-w`), appending (`--append`), checking if sorted (`--check`), and verbose output (`--verbose`). When no positional arguments are given and stdin is a TTY, the tool SHALL print help and exit 0. When no positional arguments and stdin is not a TTY, the tool SHALL read from stdin. Flag combinations MUST be validated: `-i` and `-w` are mutually exclusive, `--check` is mutually exclusive with both `-i` and `-w`, `--append` requires `-w`, and `-w` with a glob matching multiple files MUST error.

#### Scenarios

**Given** a single markdown file with default options (no flags):
**When** run as `smt file.md`, **Then** sort marked tables and output to stdout; file remains unchanged.

---

**Given** a single file with `-i` flag:
**When** run as `smt -i file.md`, **Then** sort marked tables and overwrite the original file atomically; if any error occurs, file remains unchanged.

---

**Given** a glob pattern matching 3 files with `-i` flag:
**When** run as `smt -i "docs/**/*.md"`, **Then** parse and sort all 3 files. If all succeed, write all 3 atomically. If any file has an error, abort before writing any file, exit 2.

---

**Given** input piped from stdin (not a TTY) with no positional arguments:
**When** run as `cat file.md | smt`, **Then** read from stdin, sort marked tables, output to stdout.

---

**Given** a TTY stdin with no positional arguments:
**When** run as `smt` in an interactive terminal, **Then** print help message and exit 0.

---

**Given** the `--check` flag with an already-sorted file:
**When** run as `smt --check sorted.md`, **Then** exit 0; no output is written.

---

**Given** the `--check` flag with an unsorted file:
**When** run as `smt --check unsorted.md`, **Then** exit 1; no output is written.

---

**Given** the `--check` and `--verbose` flags with an unsorted file:
**When** run as `smt --check --verbose unsorted.md`, **Then** print to stdout the location of each unsorted table (e.g., `unsorted.md:7: table is not sorted (comment at line 6)`) and exit 1.

---

**Given** `-i` and `-w` flags together:
**When** run as `smt -i file.md -w output.md`, **Then** emit error `"--in-place and --write are mutually exclusive"` and exit 2.

---

**Given** `--check` and `-i` flags together:
**When** run as `smt --check -i file.md`, **Then** emit error `"--check and --in-place are mutually exclusive"` and exit 2.

---

**Given** `-w` with a glob matching multiple files:
**When** run as `smt -w output.md "docs/**/*.md"` (glob matches 3 files), **Then** emit error `"--write cannot be used with multiple input files"` and exit 2.

---

**Given** `-w` with a single file:
**When** run as `smt -w output.md input.md`, **Then** sort input.md and write result to output.md.

---

**Given** `-w` and `--append` flags with a file:
**When** run as `smt -w output.md --append input.md`, **Then** sort input.md and append result to output.md (do not overwrite).

---

**Given** `--append` without `-w`:
**When** run as `smt --append input.md`, **Then** emit error `"--append requires --write"` and exit 2.

---

**Given** `-i` with stdin:
**When** run as `cat file.md | smt -i`, **Then** emit error `"--in-place cannot be used with stdin"` and exit 2.

---

**Given** `-w` with stdin:
**When** run as `cat input.md | smt -w output.md`, **Then** read from stdin, sort, and write to output.md (allowed).

---

**Given** a glob pattern that matches zero files:
**When** run as `smt "docs/**/*.nonexistent"`, **Then** emit error `"no files matched pattern \"docs/**/*.nonexistent\""` and exit 2.

---

### 2.4 Atomicity

**Requirement**: On ANY error (parse, sort, OR write), NO files SHALL be modified, even when using `-i` with multiple files.

This requires more than “two-phase” processing; Phase 2 must also be transactional across all files (e.g. write all temp outputs, then commit via a rename strategy that supports rollback using backups).

#### Scenarios

**Given** 3 files marked for in-place sorting with `-i`, where file #2 has a parse error (e.g., column out of range):
**When** run as `smt -i file1.md file2.md file3.md`, **Then** all 3 files are read and parsed. Phase 1 fails on file #2. No files are written. Exit code 2.

---

**Given** multiple files and a permission denied error during the write phase (e.g. in-place commit step cannot rename/persist one file):
**When** run as `smt -i file1.md file2.md file3.md`, **Then** Phase 1 succeeds (parse and sort). Phase 2 fails. The tool MUST roll back any partial commits so that all original files remain unchanged. Exit code 2.

---

**Given** 5 files to be sorted in-place, all parse successfully, all sort successfully:
**When** Phase 1 completes, **Then** Phase 2 MUST commit all 5 outputs as an all-or-nothing operation. If any individual commit step fails (e.g., permission denied on file 4), the tool MUST roll back any files already committed so all 5 files are unchanged. Exit code 2.

---

### 2.5 Error Handling

**Requirement**: All errors MUST be emitted to stderr with descriptive messages including file path and line number (where applicable). Errors in comment options, missing tables, out-of-range columns, malformed tables, file I/O, and CLI validation MUST map to exit code 2. The `--check` mode produces exit code 1 for unsorted tables (not an error, but a check result). Success is exit code 0.

#### Scenarios

**Given** a file not found:
**When** run as `smt missing.md`, **Then** emit error `"file not found: missing.md"` to stderr and exit 2.

---

**Given** a file without read permission:
**When** run as `smt /root/restricted.md` (permission denied), **Then** emit error `"permission denied: /root/restricted.md"` to stderr and exit 2.

---

**Given** a malformed table (header row without separator row):

```markdown
<!-- smt -->

| Name | Age |
| Bob | 30 |
```

**When** parsed, **Then** emit error `"malformed table at line X (missing separator row)"` and exit 2.

---

**Given** stderr output vs stdout:
**When** an error occurs, **Then** error message goes to stderr. **When** normal sorted output is produced, it goes to stdout. **When** `--check --verbose` reports unsorted tables, they go to stdout (not errors, but check output).

---

## 3. Non-Functional Requirements

### 3.1 Deployment & Runtime

- The tool MUST compile to a single static binary with NO runtime dependencies (no shared libraries, no JVM, no Python interpreter, etc.)
- The binary MUST be deployed via `cargo install` (Rust package manager)

### 3.2 Performance

- Startup time MUST be negligible (<10ms on typical hardware)
- Parsing and sorting large files (100+ tables) MUST complete in <1 second

### 3.3 Markdown Fidelity

- The tool MUST preserve markdown formatting: whitespace (both within cells and row indentation), line endings (LF vs CRLF), and column alignment MUST NOT be altered
- The tool MUST perform lossless round-trip: a markdown file without marked tables SHALL pass through unchanged
- The tool MUST preserve original cell content in output, only reordering rows

### 3.4 Sorting Guarantee

- The sort MUST be stable (Rust's `slice::sort_by`, never `sort_unstable_by`)
- Identical sort keys MUST result in deterministic output: rows that compare as equal retain their original relative order

### 3.5 Comment Parsing

- NO regex library: comment parsing MUST use hand-rolled `str` methods (`strip_prefix`, `split_whitespace`, `strip_suffix`, etc.)
- This constraint reduces compile-time cost and binary size

---

## 4. Success Criteria

Verification that each requirement is met:

1. **Comment Parsing**: All unit tests for `parser::parse_comment()` pass; test cases cover valid defaults, all option combinations, unknown keys, invalid values, out-of-range columns, comments without tables, and duplicate comments.

2. **Sorting Behavior**: All unit tests for `sorter` module pass; test cases cover numeric sort (integers, floats, negatives, non-numeric fallback), lexicographic sort (case-sensitive and insensitive), sort direction (asc, desc), stability (equal elements preserve order), and edge cases (single row, zero rows).

3. **CLI Validation**: All `cli` module tests pass; flag combination tests verify mutual exclusivity constraints, positional argument parsing, glob expansion, and error messages.

4. **Atomicity**: Integration test with `-i` and multiple files demonstrates that error in one file prevents all writes. All 3 files remain unchanged after error. Exit code 2.

5. **File I/O & Atomicity for In-Place**: Integration test for `-i` writes to a temp file in the same directory, then renames atomically. Original file is never left corrupted.

6. **Error Messages**: All error scenarios from `openspec/specs/smt/plan.md` (Error Handling) are covered by integration tests using `assert_cmd` and `predicates`. Error messages are checked for correctness (file path, line number, descriptive text). Errors go to stderr, exit code 2.

7. **Exit Codes**: Integration tests verify:
   - Exit 0 on success (sorted or in-place write)
   - Exit 1 when `--check` finds unsorted tables
   - Exit 2 for all user errors (bad CLI, parse errors, I/O errors)

8. **Markdown Preservation**: Integration test with a file containing multiple tables (some with `<!-- smt -->`, some without) and complex formatting. Sorted tables are reordered; unmarked tables and non-table content are unchanged. Output matches expected fixture.

9. **Check Mode**: Integration tests for `--check` (exit 0 if sorted, exit 1 if not), `--check --verbose` (prints unsorted table locations to stdout).

10. **No Unsafe Code**: Code review confirms zero `unsafe` blocks (except where required by platform dependencies, if any).

11. **Compile Without Warnings**: Build with `cargo build --release` produces no compiler warnings.

12. **Dependency Minimalism**: `Cargo.toml` contains only required crates (clap, thiserror, anyhow, glob, tempfile). NO regex, NO serde, NO heavy transitive deps. Binary size is acceptable for distribution.

---

## 5. Specification References

- **`openspec/specs/smt/plan.md`**: Comment syntax, CLI interface, exit codes, sorting behavior, error handling, dependencies, testing strategy
- **`openspec/specs/smt/architecture.md`**: Data structures (`Document`, `Block`, `Table`, `SortOptions`), data flow, error type hierarchy, parser state machine, atomic write strategy, check mode flow, comment parsing detail, rendering back to markdown

---

## 6. Out of Scope

The following features are explicitly NOT included in this specification and are deferred to future versions:

- Multi-column sort (sort by column 2, then by column 3 as tiebreaker)
- Column alignment/reformatting after sort
- Configuration file (`.smtrc`)
- Watch mode (`--watch`)
- Ignore patterns (`<!-- smt-ignore -->`)
- TOML/YAML frontmatter awareness
- Markdown-aware sorting (e.g., sort by link text in `[text](url)`)
- `--diff` mode
- Homebrew/cargo-binstall distribution channels
- Pre-commit hook package definition

---

## 7. Glossary

- **Marked table**: A table preceded by an `<!-- smt -->` HTML comment.
- **Unmarked table**: A table without an `<!-- smt -->` comment (passes through unchanged).
- **Sort key**: The value extracted from the sort column used for comparison.
- **Numeric sort**: Parsing cell content as `f64` for comparison; non-numeric content sorts after numeric.
- **Lexicographic sort**: Unicode string comparison, optionally case-insensitive.
- **Stable sort**: Rows with equal sort keys retain their original relative order.
- **Atomic write**: Either all files are written or none are written; no partial updates.
- **Two-phase**: Phase 1 (parse & sort in memory), Phase 2 (write to disk).
