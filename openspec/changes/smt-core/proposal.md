# Proposal: smt-core — Core Implementation of Sort Markdown Tables CLI

## Intent

Implement the complete `smt` CLI tool from scratch, a Rust-based markdown table sorting utility designed for CI pipelines and pre-commit hooks. The tool will parse markdown files, identify tables preceded by `<!-- smt -->` comments, sort them according to specified options (column, direction, case sensitivity, numeric/lexicographic), and output results to stdout, files, or in-place with guaranteed atomicity.

This change’s proposal/spec/design are migrated from the canonical specs in `openspec/specs/smt/plan.md` and `openspec/specs/smt/architecture.md` (the OpenSpec artifacts should be treated as the source of truth going forward).

## Scope

### In Scope

**Phase 1: Infrastructure & Core Modules**

- `error.rs` — Custom error enum (`SmtError`) with all error variants for CLI, parse, and I/O errors
- `cli.rs` — Clap-based argument parsing with derive macros, glob expansion, and validation of flag combinations
- `parser.rs` — Markdown parser with line-by-line state machine to extract tables, detect smt comments, parse sort options
- `sorter.rs` — Sorting logic with numeric and lexicographic comparisons, stable sort, case sensitivity, direction control
- `writer.rs` — Output handling for stdout, file writes, in-place (atomic via tempfile), and append mode

**Phase 2: Integration & Testing**

- `main.rs` — Two-phase orchestrator: (1) parse & sort all inputs, (2) write all results (atomicity guarantee)
- Unit tests for each module covering edge cases, error conditions, and option parsing
- Integration tests using `assert_cmd` and `predicates` with test fixtures
- Test fixtures in `tests/fixtures/` with `.md` input and `.expected.md` output pairs
- Cargo.toml with dependencies: clap 4.x, thiserror 2.x, anyhow 1.x, glob 0.3.x, tempfile, assert_cmd, predicates, pretty_assertions

### Out of Scope

- Multi-column sort (e.g., sort by column 2, then by column 3 as tiebreaker) — deferred to future version
- Column alignment/reformatting after sort — tool preserves original formatting
- Configuration file (`.smtrc`) — options parsed from CLI only
- Watch mode (`--watch`) — one-shot execution only
- Ignore patterns (`<!-- smt-ignore -->`) — all `<!-- smt -->` comments are processed
- TOML/YAML frontmatter awareness — tool processes entire file as-is
- Markdown-aware sorting (e.g., sort by link text in `[text](url)`) — sorts full cell content
- `--diff` mode — check mode only (exit 0/1, no output formatting)
- Homebrew/cargo-binstall distribution channels — binary available via `cargo install`
- Pre-commit hook package definition — users can reference the binary directly

## Approach

### Technical Strategy

1. **Two-Phase Atomicity**:
   - Phase 1 (Parse & Sort): Read all inputs, parse into `Document` AST, sort all tables, collect results in memory. If ANY error occurs, abort immediately (exit 2).
   - Phase 2 (Write): Only after Phase 1 succeeds for ALL files, write all results (stdout, file, or in-place). Guarantees no partial updates.

2. **No Regex for Comment Parsing**:
   - Use hand-rolled `str` methods (`strip_prefix`, `split_whitespace`, `strip_suffix`) for parsing `<!-- smt -->` comments.
   - Avoids compile-time cost of regex crate, keeps binary small and fast.

3. **Lossless Markdown Round-Trip**:
   - Parser preserves all non-table content verbatim (including whitespace, formatting, comments).
   - `Document` AST stores raw lines to enable reconstruction without loss of fidelity.
   - Output matches input 1:1 except for sorted table rows.

4. **Stable Sort Guarantee**:
   - Use `slice::sort_by` (stable), never `sort_unstable_by`.
   - Rows that compare equal preserve their original relative order.

5. **Atomic File Writes for `-i` Mode**:
   - Use `tempfile` crate's `NamedTempFile` created in the same directory as the target file.
   - Write new content to temp file, fsync, then rename (atomic on POSIX).
   - If rename fails, attempt cleanup; either way, original file is never corrupted.

### Deliverables by Phase

**Phase 1:**

- [ ] `src/error.rs` with full `SmtError` enum
- [ ] `src/cli.rs` with `Args` struct, `InputSource` enum, `OutputTarget` enum, validation logic
- [ ] `src/parser.rs` with `Document`, `Block`, `Table`, `TableRow`, `SortOptions`, and comment parsing
- [ ] `src/sorter.rs` with stable sort, numeric/lexicographic comparison, check mode support
- [ ] `src/writer.rs` with stdout, file, and in-place write handlers
- [ ] Unit tests for all above modules
- [ ] `Cargo.toml` with all required dependencies

**Phase 2:**

- [ ] `src/main.rs` orchestrator with two-phase pipeline
- [ ] Integration tests with fixtures and `assert_cmd`
- [ ] Handle `--check` mode with optional `--verbose` output
- [ ] Verify exit codes: 0 (success), 1 (check failed), 2 (user error)
- [ ] Manual testing of CLI interface and edge cases

## Affected Areas

| Area              | Impact   | Description                                                           |
| ----------------- | -------- | --------------------------------------------------------------------- |
| `src/error.rs`    | New      | Custom error enum with thiserror, all error variants for CLI/parse/IO |
| `src/cli.rs`      | New      | Clap-based argument parser with flag validation and glob expansion    |
| `src/parser.rs`   | New      | Markdown parser with state machine, comment detection, option parsing |
| `src/sorter.rs`   | New      | Stable sorting logic with numeric, lexicographic, case, direction     |
| `src/writer.rs`   | New      | Output handling for stdout, file, in-place (atomic), append           |
| `src/main.rs`     | New      | Entry point orchestrating two-phase pipeline (parse/sort, then write) |
| `tests/fixtures/` | New      | Test fixtures with `.md` inputs and `.expected.md` outputs            |
| `Cargo.toml`      | Modified | Add clap, thiserror, anyhow, glob, tempfile, assert_cmd, predicates   |

## Risks

| Risk                                                          | Likelihood | Mitigation                                                                                                            |
| ------------------------------------------------------------- | ---------- | --------------------------------------------------------------------------------------------------------------------- |
| Atomicity violation (partial file writes on error)            | Medium     | Two-phase implementation with in-memory result collection before any writes. Comprehensive error handling in Phase 1. |
| Performance regression (slow parsing for large files)         | Low        | Single-pass line-by-line parser, no regex, no unnecessary allocations. Benchmarking in integration tests.             |
| Glob expansion edge cases (empty matches, special characters) | Low        | Use standard `glob` crate; validate and error on zero matches. Test with fixtures.                                    |
| Markdown round-trip fidelity (whitespace/format loss)         | Low        | Store raw lines in `TableRow::raw` and `PlainText` blocks. Preserve input exactly.                                    |
| Comment parsing ambiguities (malformed options)               | Low        | Strict validation: unknown keys and invalid values both error out (no silent ignoring). Comprehensive test coverage.  |

## Rollback Plan

Since this is a **greenfield implementation** (building from scratch with no prior code to revert), rollback is not applicable. However:

- If build integration issues arise post-implementation, revert commit(s) that added `src/` changes and `Cargo.toml` modifications.
- If runtime issues are discovered in production, revert the commit and fall back to manual markdown sorting or existing tools.
- The feature is opt-in (marked tables only), so impact scope is limited to markdown files with `<!-- smt -->` comments.

## Dependencies

- **Rust stable toolchain** (no MSRV specified, use current stable)
- **External crates**: clap, thiserror, anyhow, glob, tempfile, assert_cmd, predicates, pretty_assertions (all Cargo.toml entries)
- **`openspec/specs/smt/plan.md`**: Complete functional specification and requirements
- **`openspec/specs/smt/architecture.md`**: Detailed module structure, data flow, and API design

## Success Criteria

- [ ] All 6 modules (`error`, `cli`, `parser`, `sorter`, `writer`, `main`) implemented and compile without warnings
- [ ] 100% of unit tests pass (error handling, option parsing, sorting, writer behavior)
- [ ] All integration tests pass (`assert_cmd` tests with fixtures, `--check` mode, exit codes)
- [ ] Manual testing confirms CLI works as specified: stdin, files, globs, `-i`, `-w`, `--append`, `--check`, `--verbose`
- [ ] Atomicity verified: error in one file prevents all writes when using `-i` with glob
- [ ] Performance acceptable: parsing and sorting large files (100+ tables) in <1 second
- [ ] Zero unsafe code (except where required by platform dependencies)
- [ ] Binary compiles to a single static executable with no runtime dependencies
