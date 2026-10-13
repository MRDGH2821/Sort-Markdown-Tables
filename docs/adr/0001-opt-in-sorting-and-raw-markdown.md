---
status: accepted
---

# Opt in to sorting and retain raw Markdown

Documentation tools must keep edits limited to the tables that authors select.
Use SMT comments to mark tables, and retain original line text while comparing cell values separately.
This keeps sorting independent of column alignment and Markdown rendering, at the cost of retaining both raw rows and comparison values.

The core design uses a small line parser and string operations for the marker grammar.
It avoids a regular-expression dependency to reduce build cost and keep the binary small.
Only data rows change position, while headers, separators, comments, and unmarked content retain their place.

Sources: [Core specification, sections 2.1, 3.3, 3.5, and 6](../../openspec/changes/archive/2026-05-09-smt-core/spec.md),
[core design, section 2.3](../../openspec/changes/archive/2026-05-09-smt-core/design.md).
Line terminators follow [ADR 0004](0004-editorconfig-line-endings.md).
