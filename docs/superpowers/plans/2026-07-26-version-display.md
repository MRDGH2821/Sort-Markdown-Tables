# Version Display Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add a version subcommand (`smt version`) and flags (`smt --version`, `smt -V`) displaying `Sort Markdown Tables v0.1.4`.

**Architecture:** Extend `clap` argument definitions in `src/cli.rs` with `name = "Sort Markdown Tables"`, `version = concat!("v", env!("CARGO_PKG_VERSION"))`, and an optional `SmtSubcommand` enum with a `Version` variant. Update `finalize_cli` / `run_with_routing` in `src/main.rs` to print version output and exit cleanly with status 0.

**Tech Stack:** Rust, clap 4.x, assert_cmd (for integration tests).

## Global Constraints

- **Exact Version Output**: `Sort Markdown Tables v0.1.4` (for version `0.1.4`)
- **Supported Commands**: `smt --version`, `smt -V`, `smt version`
- **Exit Code**: `0` on version display
- **Atomicity & Quality**: All pre-commit hooks and tests must pass. Commit messages must use `Co-authored-by` trailer.

---

### Task 1: Add Subcommand and Version Flag Parsing to `src/cli.rs` and `src/main.rs`

**Files:**

- Modify: `src/cli.rs`
- Modify: `src/main.rs`
- Test: `src/cli.rs` (unit tests)

**Interfaces:**

- Produces: `pub enum SmtSubcommand { Version }`, `pub struct Args { pub subcommand: Option<SmtSubcommand>, ... }`

- [ ] **Step 1: Write failing unit test in `src/cli.rs`**

```rust
#[test]
fn test_cli_version_subcommand_parsing() {
    let args = Args::try_parse_from(["smt", "version"]).unwrap();
    assert_eq!(args.subcommand, Some(SmtSubcommand::Version));
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test --lib cli::tests::test_cli_version_subcommand_parsing`
Expected: FAIL with missing fields/enum `SmtSubcommand`.

- [ ] **Step 3: Implement `SmtSubcommand` and `Args` updates in `src/cli.rs` & `src/main.rs`**

In `src/cli.rs`:

```rust
use clap::{Parser, Subcommand};

#[derive(Subcommand, Debug, Clone, PartialEq, Eq)]
pub enum SmtSubcommand {
    /// Display version information
    Version,
}

#[derive(Parser, Debug)]
#[command(name = "Sort Markdown Tables")]
#[command(version = concat!("v", env!("CARGO_PKG_VERSION")))]
#[command(about = "Sort Markdown Tables", long_about = None)]
pub struct Args {
    #[command(subcommand)]
    pub subcommand: Option<SmtSubcommand>,

    /// Input files or glob patterns
    pub inputs: Vec<String>,
    ...
```

In `src/main.rs`, update `run_with_routing` to handle version subcommand:

```rust
if let Some(smt::cli::SmtSubcommand::Version) = args.subcommand {
    println!("Sort Markdown Tables v{}", env!("CARGO_PKG_VERSION"));
    return 0;
}
```

- [ ] **Step 4: Run unit test to verify it passes**

Run: `cargo test --lib cli::tests::test_cli_version_subcommand_parsing`
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add src/cli.rs src/main.rs
git commit --trailer="AI-Model: gemini-3.6-flash" -m "feat(cli): add version subcommand and version attribute

Co-authored-by: Gemini 3.6 Flash via Antigravity <noreply@google.com>"
```

---

### Task 2: Add Integration Tests for `--version`, `-V`, and `version`

**Files:**

- Modify: `tests/integration_test.rs`

- [ ] **Step 1: Write integration tests in `tests/integration_test.rs`**

```rust
#[test]
fn test_version_flag_long() {
    let mut cmd = Command::cargo_bin("smt").unwrap();
    cmd.arg("--version")
        .assert()
        .success()
        .stdout(predicates::str::contains("Sort Markdown Tables v"));
}

#[test]
fn test_version_flag_short() {
    let mut cmd = Command::cargo_bin("smt").unwrap();
    cmd.arg("-V")
        .assert()
        .success()
        .stdout(predicates::str::contains("Sort Markdown Tables v"));
}

#[test]
fn test_version_subcommand() {
    let mut cmd = Command::cargo_bin("smt").unwrap();
    cmd.arg("version")
        .assert()
        .success()
        .stdout(predicates::str::contains("Sort Markdown Tables v"));
}
```

- [ ] **Step 2: Run integration tests to verify they pass**

Run: `cargo test --test integration_test test_version`
Expected: PASS

- [ ] **Step 3: Commit**

```bash
git add tests/integration_test.rs
git commit --trailer="AI-Model: gemini-3.6-flash" -m "test(cli): add integration tests for version flag and subcommand

Co-authored-by: Gemini 3.6 Flash via Antigravity <noreply@google.com>"
```
