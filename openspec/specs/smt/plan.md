# `smt` — Sort Markdown Tables: Project Plan

## 1. Project Overview

`smt` is a single-binary Rust CLI tool that sorts markdown tables in-place or to stdout. Tables are opted-in for sorting by placing an HTML comment `<!-- smt -->` on the line immediately preceding the table. The comment accepts key=value options to control sort column, order, case sensitivity, and sort type.

The tool is designed for CI pipelines and pre-commit hooks: it's fast, has zero runtime dependencies, and uses explicit exit codes to signal state.

---

## 2. Requirements Summary

| Category          | Requirement                                                |
| ----------------- | ---------------------------------------------------------- |
| Language          | Rust (stable toolchain)                                    |
| Binary name       | `smt`                                                      |
| Input             | Single file, glob pattern, or stdin                        |
| Output            | stdout (default), in-place (`-i`), or specific file (`-w`) |
| Opt-in            | Only tables preceded by `<!-- smt -->` are sorted          |
| Atomicity         | On ANY error, no files are modified (even with `-i`)       |
| CI-friendly       | `--check` mode with deterministic exit codes               |
| Zero runtime deps | Single static binary, no shared libraries needed           |

---

## 3. Comment Syntax

### Base Form

```markdown
<!-- smt -->

| Name  | Age |
| ----- | --- |
| Alice | 25  |
| Bob   | 30  |
```

### With Options

```markdown
<!-- smt column=2 order=desc case=insensitive type=lexicographic -->

| Name  | City      |
| ----- | --------- |
| Alice | Zurich    |
| Bob   | Amsterdam |
```

### Syntax Rules

- The comment MUST match the pattern: `<!-- smt [key=value ...] -->`
- Leading/trailing whitespace inside the comment is allowed: `<!--  smt  column=2  -->`
- Options are `key=value` pairs separated by one or more spaces
- No quotes around values
- Keys are case-sensitive (lowercase only)
- The comment MUST be on the line **immediately** preceding the table header row (no blank lines between)
- A comment that matches `<!-- smt ... -->` but contains unknown keys is an **error**, not silently ignored

### Option Reference

| Option   | Type    | Values                     | Default     | Description                                                                                   |
| -------- | ------- | -------------------------- | ----------- | --------------------------------------------------------------------------------------------- |
| `column` | integer | `1..N` (1-based)           | `1`         | Which column to sort by                                                                       |
| `order`  | enum    | `asc`, `desc`              | `asc`       | Sort direction                                                                                |
| `case`   | enum    | `sensitive`, `insensitive` | `sensitive` | Case sensitivity for lexicographic sort                                                       |
| `type`   | enum    | `numeric`, `lexicographic` | `numeric`   | Sort type. Numeric parses cell content as a number; non-numeric cells sort after numeric ones |

### Option Parsing Examples

```
<!-- smt -->                           → column=1, order=asc, case=sensitive, type=numeric
<!-- smt column=3 -->                  → column=3, order=asc, case=sensitive, type=numeric
<!-- smt order=desc type=lexicographic --> → column=1, order=desc, case=sensitive, type=lexicographic
<!-- smt colum=2 -->                   → ERROR: unknown option "colum"
<!-- smt column=0 -->                  → ERROR: column must be >= 1
<!-- smt column=abc -->                → ERROR: column must be an integer
<!-- smt order=ascending -->           → ERROR: invalid value "ascending" for option "order"
```

---

## 4. CLI Interface

### Usage

```
smt [OPTIONS] [FILE|GLOB...]
```

### Positional Arguments

| Argument | Description                                                             |
| -------- | ----------------------------------------------------------------------- |
| `FILE`   | Path to a single markdown file                                          |
| `GLOB`   | Glob pattern (e.g., `**/*.md`, `docs/*.md`). Multiple patterns allowed. |
| _(none)_ | Read from stdin                                                         |

When no positional arguments are given and stdin is not a TTY, `smt` reads from stdin.
When no positional arguments are given and stdin IS a TTY, print usage help and exit.

### Flags & Options

| Flag | Long         | Argument | Description                                                                                                       |
| ---- | ------------ | -------- | ----------------------------------------------------------------------------------------------------------------- |
| `-i` | `--in-place` | _(none)_ | Sort tables and write back to the original files. Works with single files and globs.                              |
| `-w` | `--write`    | `<PATH>` | Write output to a specific file path. Overwrites by default. Does NOT work with globs or multiple input files.    |
|      | `--append`   | _(none)_ | When used with `-w`, append to the output file instead of overwriting. Only valid with `-w`.                      |
|      | `--check`    | _(none)_ | Read-only mode. Exit 0 if all marked tables are already sorted. Exit 1 if any are unsorted. No output is written. |
|      | `--verbose`  | _(none)_ | In `--check` mode, print which tables are unsorted (file path + line number). Outside `--check`, no effect.       |
| `-h` | `--help`     | _(none)_ | Print help message.                                                                                               |
| `-V` | `--version`  | _(none)_ | Print version.                                                                                                    |

### Mutual Exclusivity & Validation

| Constraint                   | Error                                         |
| ---------------------------- | --------------------------------------------- |
| `--check` + `-i`             | Mutually exclusive — exit 2                   |
| `--check` + `-w`             | Mutually exclusive — exit 2                   |
| `-i` + `-w`                  | Mutually exclusive — exit 2                   |
| `-w` + glob (multiple files) | `-w` requires exactly one input file — exit 2 |
| `-w` + stdin                 | Allowed (pipe stdin, write to file)           |
| `--append` without `-w`      | `--append` requires `-w` — exit 2             |
| No input + TTY stdin         | Print help, exit 0                            |

### Output Behavior Matrix

| Input       | Flag        | Output Destination                                                                          |
| ----------- | ----------- | ------------------------------------------------------------------------------------------- |
| Single file | _(none)_    | stdout                                                                                      |
| Single file | `-i`        | Overwrite input file                                                                        |
| Single file | `-w out.md` | Write to `out.md`                                                                           |
| Glob        | _(none)_    | stdout (all files concatenated? — **No**: each file printed sequentially with no delimiter) |
| Glob        | `-i`        | Each file overwritten in-place                                                              |
| Glob        | `-w`        | **Error** (exit 2)                                                                          |
| stdin       | _(none)_    | stdout                                                                                      |
| stdin       | `-i`        | **Error** (exit 2 — can't write back to stdin)                                              |
| stdin       | `-w out.md` | Write to `out.md`                                                                           |

**Clarification on glob + stdout**: When processing multiple files without `-i`, each file's sorted content is printed to stdout sequentially. This is primarily useful for `--check` mode. For actual sorted output of multiple files, `-i` is the expected workflow.

---

## 5. Exit Codes

| Code | Meaning               | When                                                                               |
| ---- | --------------------- | ---------------------------------------------------------------------------------- |
| `0`  | Success               | Tables sorted successfully, or `--check` confirms all tables are sorted            |
| `1`  | Unsorted tables found | `--check` mode only — at least one marked table is not in sorted order             |
| `2`  | User error            | Bad arguments, invalid comment options, malformed table, column out of range, etc. |

On exit code 2, a descriptive error message is printed to stderr.

---

## 6. Sorting Behavior

### Table Structure

A markdown table consists of:

1. **Header row**: `| Col1 | Col2 | Col3 |`
2. **Separator row**: `|------|------|------|` (may include alignment markers like `:---:`)
3. **Data rows**: `| val1 | val2 | val3 |` (one or more)

### What Gets Sorted

- **Only data rows** (rows after the separator) are sorted
- The header row stays pinned at position 1
- The separator row stays pinned at position 2
- The `<!-- smt -->` comment line itself stays pinned above the header

### Column Indexing

- 1-based (as specified in the `column` option)
- Column 1 = first column in the table
- Leading/trailing whitespace in cell content is trimmed before comparison but preserved in output
- If the table has N columns and `column > N`, that's an error (exit 2)

### Numeric Sort (`type=numeric`)

- Parse cell content as `f64`
- Cells that fail to parse as a number sort **after** all numeric cells (stable relative order among non-numeric cells is preserved)
- Leading/trailing whitespace is stripped before parsing
- Supports negative numbers and decimals: `-3.14`, `0`, `42`, `1000.5`
- Does NOT support: thousand separators (`1,000`), scientific notation (`1e3`), currency symbols (`$10`). These are treated as non-numeric.

### Lexicographic Sort (`type=lexicographic`)

- Standard Unicode string comparison
- `case=insensitive`: convert to lowercase (using `.to_lowercase()`) before comparison, but preserve original case in output
- `case=sensitive`: compare as-is

### Stability

- The sort MUST be stable. Rows that compare as equal retain their original relative order.
- Rust's `sort_by` is stable, so use `slice::sort_by`, not `sort_unstable_by`.

### Multi-Table Handling

- A single file may contain multiple `<!-- smt -->` comments, each controlling the table that immediately follows it
- Tables without a preceding `<!-- smt -->` comment are left completely untouched
- Each table is sorted independently according to its own comment's options

### Whitespace Preservation

- Leading/trailing whitespace in cells is preserved in output
- Column alignment (padding) in the original table is preserved as-is — the tool does NOT reformat column widths
- The tool preserves line endings as found in the input (LF vs CRLF)

---

## 7. Error Handling

### Error Conditions

| Condition                                              | Message (example)                                                                     | Exit Code |
| ------------------------------------------------------ | ------------------------------------------------------------------------------------- | --------- |
| `<!-- smt -->` with no table following                 | `error: smt comment at line 5 is not followed by a table`                             | 2         |
| `column=99` on a 3-column table                        | `error: column 99 is out of range (table has 3 columns) at line 7`                    | 2         |
| Unknown option key                                     | `error: unknown option "colum" in smt comment at line 3`                              | 2         |
| Invalid option value                                   | `error: invalid value "ascending" for option "order" at line 3 (expected: asc, desc)` | 2         |
| Two consecutive `<!-- smt -->` comments before a table | `error: duplicate smt comment at line 4 (previous at line 3)`                         | 2         |
| `column=0` or negative                                 | `error: column must be >= 1 in smt comment at line 3`                                 | 2         |
| `column=abc` (not an integer)                          | `error: column must be a positive integer in smt comment at line 3`                   | 2         |
| `-w` with multiple input files                         | `error: --write cannot be used with multiple input files`                             | 2         |
| `--check` with `-i`                                    | `error: --check and --in-place are mutually exclusive`                                | 2         |
| `--check` with `-w`                                    | `error: --check and --write are mutually exclusive`                                   | 2         |
| `-i` with `-w`                                         | `error: --in-place and --write are mutually exclusive`                                | 2         |
| `--append` without `-w`                                | `error: --append requires --write`                                                    | 2         |
| `-i` with stdin                                        | `error: --in-place cannot be used with stdin`                                         | 2         |
| Input file not found                                   | `error: file not found: path/to/file.md`                                              | 2         |
| Input file not readable                                | `error: permission denied: path/to/file.md`                                           | 2         |
| Glob pattern matches zero files                        | `error: no files matched pattern "docs/**/*.txt"`                                     | 2         |
| Malformed table (e.g., no separator row)               | `error: malformed table at line 7 (missing separator row)`                            | 2         |

### Atomicity Rule

**On ANY error, no files are modified.** Even when using `-i` with a glob that matches 100 files, if file #37 has an error, NONE of the 100 files are written.

Implementation strategy:

1. **Phase 1 — Parse & Sort**: Read all input files, parse all tables, sort all marked tables, collect results in memory
2. **Phase 2 — Write**: Only after Phase 1 completes successfully for ALL files, write all results to disk

This two-phase approach guarantees atomicity.

### Error Output

- All errors go to **stderr**
- Normal sorted output goes to **stdout**
- In `--check --verbose` mode, unsorted table reports go to **stdout** (they're the expected output, not errors)

---

## 8. Architecture

### Module Breakdown

```
src/
├── main.rs          # Entry point, orchestrates the pipeline
├── cli.rs           # Clap argument definitions and validation
├── parser.rs        # Markdown parsing: comment detection, table extraction
├── sorter.rs        # Sorting logic for table rows
├── writer.rs        # Output handling: stdout, file, in-place
└── error.rs         # Custom error types with thiserror
```

### Module Responsibilities

#### `cli` Module

- Define `Args` struct with clap derive macros
- Mutual exclusivity validation (clap groups or manual post-parse checks)
- Glob expansion (convert glob patterns to file lists)
- Detect stdin vs file input
- Validate: `--append` requires `-w`, `-w` incompatible with multiple files, etc.

#### `parser` Module

- **Input**: Raw markdown string (+ source file path for error messages)
- **Output**: `Document` struct — a sequence of `Block` variants (plain text, smt-comment + table pairs, standalone tables)
- Responsibilities:
  - Detect `<!-- smt ... -->` comments using exact pattern matching
  - Parse comment options into `SortOptions` struct
  - Validate option keys and values
  - Extract table boundaries (header, separator, data rows)
  - Detect error conditions (comment without table, duplicate comments, column out of range)
  - Preserve all non-table content verbatim

#### `sorter` Module

- **Input**: Table data rows + `SortOptions`
- **Output**: Sorted data rows
- Responsibilities:
  - Extract sort-key from the specified column for each row
  - Apply numeric or lexicographic comparison
  - Apply case sensitivity
  - Apply sort direction
  - Stable sort

#### `writer` Module

- **Input**: Processed `Document` (with sorted tables) + output configuration
- **Output**: Write to stdout, file, or in-place
- Responsibilities:
  - Render `Document` back to markdown string
  - Handle file I/O (create, overwrite, append)
  - For in-place: write to temp file then rename (atomic file replacement)

#### `error` Module

- Custom error enum with `thiserror`
- Variants for each error category: parse errors, CLI errors, I/O errors
- Each variant carries context: file path, line number, descriptive message
- Map to exit codes (all map to exit code 2; only `--check` produces exit code 1, handled in `main.rs`)

### Data Flow

```
Input(s) ──→ cli::parse_args()
               │
               ▼
        ┌─────────────┐
        │  For each    │
        │  input file  │──→ parser::parse(content, path)
        │  (or stdin)  │         │
        └─────────────┘         ▼
                         Document { blocks }
                                │
                                ▼
                         sorter::sort_tables(&mut document)
                                │
                                ▼
                         Sorted Document
                                │
                         (collect all results)
                                │
                                ▼
                   ┌── --check? ──→ compare original vs sorted
                   │                    │
                   │              exit 0 or 1
                   │
                   └── write ──→ writer::write(document, config)
                                      │
                                stdout / file / in-place
```

---

## 9. Rust Crate Dependencies

### Required

| Crate       | Version | Feature  | Purpose                                        |
| ----------- | ------- | -------- | ---------------------------------------------- |
| `clap`      | `4.x`   | `derive` | CLI argument parsing with derive macros        |
| `thiserror` | `2.x`   | —        | Ergonomic custom error types                   |
| `anyhow`    | `1.x`   | —        | Error propagation in main, context-rich errors |
| `glob`      | `0.3.x` | —        | File glob pattern expansion                    |

### Optional / Consider

| Crate                  | Purpose                                         | Decision                                                                                                                                                                           |
| ---------------------- | ----------------------------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `regex`                | Parsing `<!-- smt ... -->` comments             | **Skip** — the comment format is simple enough to parse with `str::strip_prefix`, `split_whitespace`, and `str::strip_suffix`. Hand-rolling avoids the compile-time cost of regex. |
| `tempfile`             | Atomic file writes (write to temp, then rename) | **Recommended** for `-i` mode. Ensures no partial writes on crash.                                                                                                                 |
| `atty` / `is-terminal` | Detect if stdin is a TTY                        | **Use `std::io::IsTerminal`** (stabilized in Rust 1.70). No crate needed.                                                                                                          |

### Dependency Philosophy

Zero unnecessary dependencies. The binary should compile fast and produce a small static binary. Every crate must justify its inclusion.

---

## 10. Testing Strategy

### Unit Tests

| Module   | What to Test                                                                    |
| -------- | ------------------------------------------------------------------------------- |
| `parser` | Comment detection (valid, invalid, with options, without)                       |
| `parser` | Option parsing (all valid combos, unknown keys, invalid values, missing values) |
| `parser` | Table extraction (header, separator, data rows, edge cases)                     |
| `parser` | Multi-table documents                                                           |
| `parser` | Comment without following table (error)                                         |
| `parser` | Duplicate comments (error)                                                      |
| `parser` | Column out of range (error)                                                     |
| `sorter` | Numeric sort (integers, floats, negatives, non-numeric fallback)                |
| `sorter` | Lexicographic sort (case sensitive and insensitive)                             |
| `sorter` | Sort direction (asc, desc)                                                      |
| `sorter` | Stability (equal elements preserve order)                                       |
| `sorter` | Single-row table (no-op)                                                        |
| `sorter` | Empty data rows (no-op)                                                         |
| `writer` | Markdown rendering preserves formatting                                         |

### Integration Tests

| Test             | Description                                                   |
| ---------------- | ------------------------------------------------------------- |
| Basic sort       | `smt file.md` sorts and prints to stdout                      |
| In-place         | `smt -i file.md` modifies file                                |
| Write to file    | `smt -w out.md file.md` creates out.md                        |
| Append           | `smt -w out.md --append file.md` appends                      |
| Stdin            | `cat file.md \| smt` works                                    |
| Check pass       | `smt --check sorted.md` exits 0                               |
| Check fail       | `smt --check unsorted.md` exits 1                             |
| Check verbose    | `smt --check --verbose unsorted.md` prints locations, exits 1 |
| Glob             | `smt -i "tests/**/*.md"` processes multiple files             |
| Atomicity        | Error in one file prevents all writes                         |
| Mutual exclusion | `smt --check -i file.md` exits 2                              |
| No smt comment   | File without comments passes through unchanged                |
| Multiple tables  | File with 3 tables, 2 with comments — only 2 sorted           |

### Test Fixture Strategy

- `tests/fixtures/` directory with `.md` files as inputs
- Corresponding `.expected.md` files for expected outputs
- Test runner reads input, runs `smt`, compares output to expected
- Use `assert_cmd` crate for CLI integration tests
- Use `predicates` crate for output assertions

### Recommended Test Crates

| Crate               | Purpose                                                       |
| ------------------- | ------------------------------------------------------------- |
| `assert_cmd`        | Run the compiled binary and assert on stdout/stderr/exit code |
| `predicates`        | Rich assertions for string matching                           |
| `tempfile`          | Create temporary files/dirs for in-place and write tests      |
| `pretty_assertions` | Better diff output on test failures                           |

---

## 11. Future Considerations

These are explicitly **NOT in scope** for v1 but are noted for future reference:

| Feature                         | Notes                                                               |
| ------------------------------- | ------------------------------------------------------------------- |
| Multi-column sort               | `column=2,3` — sort by column 2, then by column 3 as tiebreaker     |
| Custom sort key                 | `key=regex:...` — extract sort key via regex                        |
| Column alignment / reformatting | Auto-pad columns to align pipes after sorting                       |
| Config file                     | `.smtrc` or similar for project-wide defaults                       |
| Watch mode                      | `smt --watch` — re-sort on file changes                             |
| Ignore patterns                 | `<!-- smt-ignore -->` to explicitly skip a table                    |
| TOML/YAML frontmatter awareness | Skip frontmatter when parsing                                       |
| Markdown link/image awareness   | Sort by link text, not full `[text](url)` syntax                    |
| Pre-commit hook package         | Published hook definition for `.pre-commit-config.yaml`             |
| Homebrew / cargo-binstall       | Distribution channels                                               |
| `--diff` mode                   | Show a diff of what would change (like `rustfmt --check` with diff) |
| Comment placement flexibility   | Allow comment 2 lines above, or after the table                     |
