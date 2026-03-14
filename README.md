# Sort Markdown Tables

[![Copier](https://img.shields.io/endpoint?url=https://raw.githubusercontent.com/copier-org/copier/refs/heads/master/img/badge/black-badge.json)](https://github.com/copier-org/copier)

A fast, zero-dependency Rust CLI tool to sort markdown tables. Tables are opted-in via `<!-- smt -->` HTML comments.

## Features

- **Opt-in sorting** — Only tables marked with `<!-- smt -->` are sorted
- **Multiple sort modes** — Numeric, lexicographic (case-sensitive/insensitive), ascending/descending
- **Atomic writes** — Two-phase pipeline ensures zero file modifications on any error
- **Glob support** — Process multiple files with patterns like `docs/**/*.md`
- **Check mode** — Validate if files are sorted without modifying them
- **In-place editing** — Modify files directly or output to stdout
- **Fast** — Startup <1ms, processes 100+ tables in <1ms
- **Small binary** — 1.2MB (release build, fully optimized)

## Installation

### From source (requires Rust 1.70+)

```bash
cargo install --path .
```

Or download a pre-built binary from [releases](https://github.com/MRDGH2821/Sort-Markdown-Tables/releases).

## Usage

### Basic: Sort a file and print to stdout

```bash
smt documents/file.md
```

### Sort in-place (modifies file)

```bash
smt -i documents/file.md
```

### Check if files are sorted (don't modify)

```bash
smt --check documents/file.md
echo $?  # Exit 0 = sorted, 1 = unsorted, 2 = error
```

### Save output to a different file

```bash
smt documents/file.md -o documents/file-sorted.md
```

### Process multiple files with glob patterns

```bash
smt -i "docs/**/*.md"
```

### Use with stdin/stdout

```bash
cat documents/file.md | smt | tee output.md
```

## Table Format

To opt-in to sorting, place `<!-- smt -->` immediately before a table:

```markdown
Some introduction text.

<!-- smt -->
| Name   | Age | Score |
|--------|-----|-------|
| Alice  | 30  | 95    |
| Bob    | 25  | 87    |
| Charlie| 28  | 92    |

Rest of the document...
```

### Valid Sort Configurations

Use attributes in the comment to configure sorting:

```markdown
<!-- smt mode=numeric column=1 direction=ascending case=insensitive -->
| A | B |
|---|---|
| 3 | z |
| 1 | x |
```

| Attribute | Options | Default | Notes |
|-----------|---------|---------|-------|
| `mode` | `numeric`, `lexicographic` | `lexicographic` | Numeric handles decimals, NaN safely |
| `column` | `1`, `2`, `3`... | `1` | 1-indexed; first data column only |
| `direction` | `ascending`, `descending` | `ascending` | Sort order |
| `case` | `sensitive`, `insensitive` | `sensitive` | Only affects lexicographic mode |

### Tables without `<!-- smt -->` are untouched

```markdown
| This table | is left | alone |
|-----------|---------|-------|
| 3 | z |
| 1 | x |
```

## Exit Codes

- `0` — Success (file(s) sorted or already sorted in check mode)
- `1` — Check failed (file is unsorted)
- `2` — Error (invalid arguments, file not found, I/O error, etc.)

## Examples

### Pre-commit hook integration

```yaml
# .pre-commit-config.yaml
repos:
  - repo: local
    hooks:
      - id: smt
        name: Sort Markdown Tables
        entry: smt --check
        language: system
        types: [markdown]
```

### CI/CD validation

```bash
#!/bin/bash
smt --check "docs/**/*.md"
if [ $? -eq 1 ]; then
  echo "Markdown tables are unsorted. Run: smt -i 'docs/**/*.md'"
  exit 1
fi
```

### Bulk formatting

```bash
# Sort all markdown files in a directory
smt -i "**/*.md"
```

## Implementation Details

### Architecture

The CLI implements a two-phase pipeline for atomicity:

1. **Parse Phase** — Read all files, parse markdown, extract tables, detect `<!-- smt -->` comments
2. **Sort Phase** — Validate all tables, sort by configured column/mode
3. **Write Phase** — Only if both phases succeed, write results atomically using temporary files

If any error occurs in phases 1-2, no files are modified.

### Algorithm

- **Stable sort guarantee** — Uses Rust's `sort_by()` (never `sort_unstable_by()`)
- **Numeric comparison** — Parses as `f64`, handles `NaN`/`Infinity` safely
- **String comparison** — UTF-8 safe, respects locale via case sensitivity flag
- **Comment parsing** — Hand-rolled (no regex), validates attributes

### Dependencies

- `clap` (4.x) — CLI argument parsing with derive macro
- `thiserror` (2.x) — Error type definitions
- `anyhow` (1.x) — Error context propagation
- `glob` (0.3.x) — File pattern globbing
- `tempfile` (3.x) — Atomic file writes

Zero external dependencies beyond those used.

## Testing

The project includes comprehensive testing:

- **106 unit tests** — All modules (error, cli, parser, sorter, writer)
- **9 integration tests** — Main pipeline and orchestration
- **20 integration tests** — End-to-end with fixtures

All 135 tests pass on debug and release builds.

Run tests:

```bash
cargo test              # Run all tests
cargo test --release   # Run in release mode
cargo test -- --nocapture  # Show output
```

## Performance

- **Startup time** — <1ms (measured with `time smt --help`)
- **Processing** — 100+ tables in <1ms
- **Binary size** — 1.2MB (fully optimized release build)
- **Memory** — No unbounded allocations; streaming parsing

## Contributing

See [ARCHITECTURE.md](.ai/ARCHITECTURE.md) for module structure and [PLAN.md](.ai/PLAN.md) for full requirements.

### Development

```bash
# Format code
cargo fmt

# Run linter
cargo clippy --all-targets

# Run all tests
cargo test

# Build release binary
cargo build --release
```

## License

See LICENSE file.

## Changelog

See [CHANGELOG.md](CHANGELOG.md) for version history, generated automatically using [cocogitto](https://github.com/cocogitto/cocogitto).
