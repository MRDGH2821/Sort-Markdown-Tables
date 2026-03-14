# AGENTS Instructions

This file provides guidance for AI coding assistants working with this project.

The primary guidelines exist in `.ai/AGENTS_GLOBAL.md`.
Always refer to this file for general instructions and best practices.

Project specific instructions will be documented here.

## Project: `smt` (Sort Markdown Tables)

A Rust CLI tool that sorts markdown tables opted-in via `<!-- smt -->` HTML comments.

### Key Documents

- `.ai/PLAN.md` — Full project plan with all requirements, CLI interface, sorting behavior, error handling, testing strategy
- `.ai/ARCHITECTURE.md` — Module dependency diagram, data flow, data structures (Rust code), parser state machine, error type hierarchy

### Architecture Overview

```
src/
├── main.rs      # Entry point, orchestrates pipeline
├── cli.rs       # Clap args, validation, glob expansion
├── parser.rs    # Markdown parsing, comment detection, table extraction
├── sorter.rs    # Sort logic (numeric, lexicographic, case, direction)
├── writer.rs    # Output: stdout, file, in-place (atomic writes)
└── error.rs     # SmtError enum with thiserror
```

### Critical Rules

- **Atomicity**: On ANY error, no files are modified (two-phase: parse+sort all, then write all)
- **Stable sort**: Use `sort_by`, never `sort_unstable_by`
- **Exit codes**: 0=success, 1=unsorted (--check only), 2=user error
- **Comment must immediately precede table** (no blank lines between)
- **Tables without `<!-- smt -->` are left untouched**

### Crate Dependencies

- `clap` (4.x, derive) — CLI parsing
- `thiserror` (2.x) — Error types
- `anyhow` (1.x) — Error propagation
- `glob` (0.3.x) — File globbing
- `tempfile` — Atomic writes for `-i` mode
- NO `regex` — comment parsing is hand-rolled with `str` methods

### Testing

- Unit tests per module (parser, sorter, writer)
- Integration tests with `assert_cmd` + `predicates`
- Test fixtures in `tests/fixtures/` with `.md` + `.expected.md` pairs
