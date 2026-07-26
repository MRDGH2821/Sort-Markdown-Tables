# Version Display Design Specification

## Overview

Provide version reporting capability for `smt` via both `--version` / `-V` CLI flags and a `version` subcommand.

## Requirements

1. **Output Format**:
   ```text
   Sort Markdown Tables v<VERSION>
   ```
   For version `0.1.4`, the exact output is:
   ```text
   Sort Markdown Tables 
   v0.1.4
   ```

2. **CLI Invocations**:
   - `smt --version`
   - `smt -V`
   - `smt version`

3. **Exit Code**:
   - `0` on successful execution of version display.

## Technical Design

### `src/cli.rs`

Update `Args` struct derive attributes and subcommand definitions:

- Configure `clap` attributes:
  - `name = "Sort Markdown Tables"`
  - `version = concat!("v", env!("CARGO_PKG_VERSION"))`
- Support both `--version` / `-V` flags (via `clap` built-in version support) and explicit `version` subcommand.

### Subcommand Support

Define `SmtSubcommand` enum:
```rust
#[derive(Subcommand, Debug, Clone, PartialEq, Eq)]
pub enum SmtSubcommand {
    /// Display version information
    Version,
}
```

In `Args`:
```rust
#[command(subcommand)]
pub subcommand: Option<SmtSubcommand>,
```

When `subcommand` is `Some(SmtSubcommand::Version)`, output `Sort Markdown Tables v<VERSION>` and exit with code `0`.

## Testing Strategy

1. **Unit Tests (`src/cli.rs`)**:
   - Verify parsing of `--version`, `-V`, and `version` subcommand.
2. **Integration Tests (`tests/integration_test.rs`)**:
   - Run `smt --version`, `smt -V`, and `smt version` via `assert_cmd` and assert exact output `Sort Markdown Tables v0.1.4` and exit code `0`.
