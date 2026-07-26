# Task 1 Brief: Add Subcommand and Version Flag Parsing to `src/cli.rs` and `src/main.rs`

**Files to modify:**

- `src/cli.rs`
- `src/main.rs`

**Requirements:**

1. In `src/cli.rs`:
   - Add `#[derive(Subcommand, Debug, Clone, PartialEq, Eq)]` enum `SmtSubcommand` with `Version` variant (`/// Display version information`).
   - Add `#[command(subcommand)] pub subcommand: Option<SmtSubcommand>` to `Args` struct.
   - Update `Args` derive macro attributes:
     - `#[command(name = "Sort Markdown Tables")]`
     - `#[command(version = concat!("v", env!("CARGO_PKG_VERSION")))]`
     - `#[command(about = "Sort Markdown Tables", long_about = None)]`
   - Add unit test `test_cli_version_subcommand_parsing` in `src/cli.rs`.
2. In `src/main.rs`:
   - In `run_with_routing` (or CLI parsing entry point in `main.rs`), check if `routing` or `args.subcommand` is `Some(SmtSubcommand::Version)`.
   - If `SmtSubcommand::Version` is present, print `Sort Markdown Tables v0.1.4` (using `env!("CARGO_PKG_VERSION")`) to stdout and return exit code `0`.

**Testing:**
Run `cargo test --lib cli::tests::test_cli_version_subcommand_parsing` and verify it passes.

**Commit:**
Commit changes with message:

```txt
feat(cli): add version subcommand and version attribute

Co-authored-by: Gemini 3.6 Flash via Antigravity <noreply@google.com>
```

Trailer: `AI-Model: gemini-3.6-flash`
