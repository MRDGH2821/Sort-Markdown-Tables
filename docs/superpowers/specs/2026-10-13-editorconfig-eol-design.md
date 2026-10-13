# Design Specification: EditorConfig Line Endings

**Date**: 2026-10-13
**Status**: Approved for review
**Topic**: Respect the effective EditorConfig `end_of_line` setting for every output path while preserving mixed input line endings when no override applies.

---

## 1. Overview

Issue [#1](https://github.com/MRDGH2821/Sort-Markdown-Tables/issues/1) reports that `smt` currently detects one newline style per document and ignores EditorConfig. The change must resolve the setting for the path being written and preserve each original line terminator when the effective setting is absent or `unset`.

The `editorconfig-parser` crate can parse a single config, match its sections against a path, and represent `unset`, but it does not search ancestor directories or merge configs. The application will perform that bounded filesystem traversal and merge; the crate will provide config parsing, path matching, and typed `end_of_line` values.

## 2. Resolution Behavior

For an output path, search `.editorconfig` files from the path's parent directory toward the filesystem root. A Git worktree root is not a stopping point. Stop only after processing a config whose preamble has `root = true`, or at filesystem root.

Apply the matching section values in EditorConfig order: ancestor configs before descendant configs, and sections from top to bottom within each config. A later `lf`, `crlf`, or `cr` value overrides an earlier value. `unset` clears the effective override, so the renderer preserves the original terminator of each input line.

Use the relevant path for resolution:

- In-place writes and file input sent to stdout: input file path.
- `--write PATH`, including stdin input: destination path.
- Stdin sent to stdout: no config lookup; preserve input terminators.

## 3. Document and Rendering Behavior

The parser must preserve line content and its individual terminator (`LF`, `CRLF`, `CR`, or no terminator at EOF). Sorting must move row content and its terminator together. When EditorConfig supplies an EOL override, render every line using that terminator; otherwise emit each preserved terminator unchanged. Retain the existing final-newline behavior.

The resolver returns only the effective EOL override needed by this feature. It does not apply unrelated EditorConfig properties.

## 4. Rust Tooling

Add the stable Rust toolchain to `mise.toml`, including the standard Cargo, rustc, rustfmt, and Clippy components. Add mise tasks matching the existing CI commands for Rust tests and Clippy. Avoid adding a second Rust toolchain/version policy: CI already uses stable.

## 5. Verification

Tests must cover mixed and CR-only input, all three configured EOL values, `unset`, nested config precedence, `root = true`, ancestor lookup above a worktree root, and output-path routing for in-place, file-to-stdout, `--write`, and stdin-to-stdout.

Run the affected Rust tests during development, then `cargo test --all-features`, the integration test target, Clippy with warnings denied, and a real CLI smoke run. Verify the new mise tasks invoke the corresponding Cargo commands.
