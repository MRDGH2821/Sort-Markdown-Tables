# Task 2 Report: Add Integration Tests for Version Flag and Subcommand

**Status**: DONE

## Summary
- Added 3 integration tests in `tests/integration_test.rs`:
  1. `test_version_flag_long`: Tests `smt --version`, verifies exit code 0 and stdout contains `"Sort Markdown Tables v"`.
  2. `test_version_flag_short`: Tests `smt -V`, verifies exit code 0 and stdout contains `"Sort Markdown Tables v"`.
  3. `test_version_subcommand`: Tests `smt version`, verifies exit code 0 and stdout contains `"Sort Markdown Tables v"`.

## Verification Results
- Ran `cargo test --test integration_test test_version`:
  `test test_version_flag_short ... ok`
  `test test_version_flag_long ... ok`
  `test test_version_subcommand ... ok`
  `test result: ok. 3 passed; 0 failed`
- Ran `cargo test`:
  `test result: ok. 121 passed` (unit tests)
  `test result: ok. 10 passed` (main/cli unit tests)
  `test result: ok. 49 passed` (integration tests)
  All 180 tests passed cleanly.
