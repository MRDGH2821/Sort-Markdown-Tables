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

### Git Workflow: Commit Scopes

**CRITICAL**: Before committing code, ensure the commit scope exists in `cog.toml`.

**Process** (required before EVERY feature commit):
1. Check `cog.toml` scopes list for your commit scope
2. If missing → add it alphabetically to the scopes array
3. Commit `cog.toml` first with message: `chore(cocogitto): add {scope} scope`
4. Then commit your code changes with the proper scope

**Example**:
```bash
# 1. Add scope to cog.toml if missing
edit cog.toml  # add "sorter" to scopes list

# 2. Commit scope update
git add cog.toml
git commit -m "chore(cocogitto): add sorter scope"

# 3. Commit actual code
git add src/sorter.rs
git commit -m "feat(sorter): implement table sorting..."
```

**Current scopes** (in `cog.toml`):
- ai, cocogitto, copier, cspell, github, jscpd, megalinter, parser, pre-commit, prettier, sorter, smt, treefmt, version, vscode, zed

Before implementing **writer** or **main**, ensure scopes are added.

### Commit Footer: AI Model Signoff

**REQUIRED**: Every commit created by an AI agent MUST include a trailer footer with the AI model name.

**Format** (using git trailer):
```bash
git commit --trailer="AI-Model: {model-name}" -m "feat(scope): description..."
```

**Git trailer output format**:
```
feat(sorter): implement table sorting logic

- Detailed description of changes
- More bullet points as needed

AI-Model: claude-haiku-4.5
```

**Purpose**: Provides transparency about which AI model contributed to each commit (for compliance, attribution, and debugging).

**Example**:
```bash
git commit --trailer="AI-Model: claude-haiku-4.5" -m "feat(sorter): implement table sorting with numeric/lexicographic comparators

- Numeric mode parses as f64 with proper NaN handling
- Lexicographic comparison with case-sensitivity toggle
- Stable sort guarantee using sort_by (never sort_unstable_by)
- 34 unit tests covering all scenarios"
```

Git automatically formats the trailer at the end of the commit message.
