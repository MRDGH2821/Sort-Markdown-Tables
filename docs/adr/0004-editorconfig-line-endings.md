---
status: accepted
---

# Resolve line endings by output path

One line-ending style per document cannot preserve mixed input or honor the destination's EditorConfig rules.
Retain each original line terminator, move it with its row, and apply the effective `end_of_line` override when one exists.
This requires per-line terminators and configuration lookup in exchange for preserving mixed endings and matching the output path's policy.

Use the input path for in-place output and file input sent to stdout, and use the destination path for `--write`.
Stdin sent to stdout has no path to resolve.
Search ancestor configurations until `root = true` or the filesystem root, including ancestors above a Git worktree.

Matching descendant values override ancestor values, and later sections override earlier sections within a configuration.
An `unset` value clears the override, so original terminators remain in use.
This feature covers only `end_of_line` and retains the existing behavior for the final newline.

The source design is labeled "Approved for review".
This record captures the implemented line-ending choice, without changing the approval status of that source document.

Source: [EditorConfig design, 2026-10-13](../superpowers/specs/2026-10-13-editorconfig-eol-design.md).
Implementation references: [resolver](../../src/editorconfig.rs), [parser](../../src/parser.rs), and [writer](../../src/writer.rs).
