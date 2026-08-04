# Architecture

## Pipeline

`smt` implements a two-phase pipeline for atomicity:

1. **Parse Phase** — Read all files, parse markdown, extract tables, detect `<!-- smt -->` comments
2. **Sort Phase** — Validate all tables, sort by configured column/mode
3. **Write Phase** — Only if both phases succeed, write results atomically using temporary files

If any error occurs in phases 1–2, no files are modified.

## Algorithm

- **Stable sort guarantee** — Uses Rust's `sort_by()` (never `sort_unstable_by()`)
- **Numeric comparison** — Parses as `f64`, handles `NaN`/`Infinity` safely
- **String comparison** — UTF-8 safe, respects locale via case sensitivity flag
- **Comment parsing** — Hand-rolled (no regex), validates attributes

## Module Layout

```
src/
├── main.rs      # Entry point, orchestrates pipeline
├── cli.rs       # Clap args, validation, glob expansion
├── parser.rs    # Markdown parsing, comment detection, table extraction
├── sorter.rs    # Sort logic (numeric, lexicographic, case, direction)
├── writer.rs    # Output: stdout, file, in-place (atomic writes)
└── error.rs     # SmtError enum with thiserror
```

## Dependencies

| Crate       | Version | Purpose                          |
| ----------- | ------- | -------------------------------- |
| `clap`      | 4.x     | CLI argument parsing with derive |
| `thiserror` | 2.x     | Error type definitions           |
| `anyhow`    | 1.x     | Error context propagation        |
| `glob`      | 0.3.x   | File pattern globbing            |
| `tempfile`  | 3.x     | Atomic file writes               |
