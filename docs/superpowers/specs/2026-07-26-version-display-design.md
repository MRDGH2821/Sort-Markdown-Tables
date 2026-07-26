# Version Display Design Specification

## Overview

Provide version reporting capability for `smt` via `--version` / `-V` CLI flags.

## Requirements

1. **Output Format**:

   ```text
   Sort Markdown Tables v<VERSION>
   ```

   For version `0.1.4`, the exact output is:

   ```text
   Sort Markdown Tables v0.1.4
   ```

2. **CLI Invocations**:
   - `smt --version`
   - `smt -V`

3. **Exit Code**:
   - `0` on successful execution of version display.

## Technical Design

### `src/cli.rs`

Update `Args` struct derive attributes:

- Configure `clap` attributes:
  - `name = "Sort Markdown Tables"`
  - `version = concat!("v", env!("CARGO_PKG_VERSION"))`

## Testing Strategy

1. **Integration Tests (`tests/integration_test.rs`)**:
   - Run `smt --version` and `smt -V` via `assert_cmd` and assert exact output `Sort Markdown Tables v0.1.4` and exit code `0`.
