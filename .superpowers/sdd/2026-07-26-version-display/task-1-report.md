# Task 1 Implementation Report: Add Subcommand and Version Flag Parsing

## Summary
- Implemented `SmtSubcommand` enum with `Version` variant in `src/cli.rs`.
- Added `subcommand: Option<SmtSubcommand>` field to `Args` struct in `src/cli.rs`.
- Configured `clap` attributes `#[command(name = "Sort Markdown Tables")]`, `#[command(version = concat!("v", env!("CARGO_PKG_VERSION")))]`, and `#[command(about = "Sort Markdown Tables", long_about = None)]` on `Args`.
- Updated `finalize_cli` and `parse_args` to pass `subcommand` to caller.
- Handled `SmtSubcommand::Version` in `run_with_routing` in `src/main.rs` to print `Sort Markdown Tables v0.1.4` to stdout and exit with status `0`.
- Applied TDD: added failing unit test `test_cli_version_subcommand_parsing`, implemented minimal logic, and verified test passing.
- Verified all 177 unit and integration tests pass without regression.
- Committed changes with Conventional Commits, `Co-authored-by` trailer, and `AI-Model: gemini-3.6-flash` trailer.

## Status
DONE
