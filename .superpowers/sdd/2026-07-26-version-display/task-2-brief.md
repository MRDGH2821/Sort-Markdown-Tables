# Task 2 Brief: Add Integration Tests for `--version`, `-V`, and `version`

**Files to modify:**
- `tests/integration_test.rs`

**Requirements:**
Write 3 integration tests using `assert_cmd` / `Command::cargo_bin("smt")`:
1. `test_version_flag_long`: Invokes `smt --version`, asserts success (exit 0) and stdout contains `"Sort Markdown Tables v"`.
2. `test_version_flag_short`: Invokes `smt -V`, asserts success (exit 0) and stdout contains `"Sort Markdown Tables v"`.
3. `test_version_subcommand`: Invokes `smt version`, asserts success (exit 0) and stdout contains `"Sort Markdown Tables v"`.

**Testing:**
Run `cargo test --test integration_test test_version` and verify all 3 tests pass.
Run `cargo test` to ensure full test suite passes.

**Commit:**
Commit changes with message:
```txt
test(cli): add integration tests for version flag and subcommand

Co-authored-by: Gemini 3.6 Flash via Antigravity <noreply@google.com>
```
Trailer: `AI-Model: gemini-3.6-flash`
