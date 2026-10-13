---
status: accepted
---

# Prepare all inputs before replacing an in-place batch

CI and formatting hooks can select many files in one invocation.
Process every input successfully before writing output, so that a later parse or sort error cannot leave earlier files changed.
This uses more memory than processing and writing each file immediately.

For in-place output, prepare temporary files beside their destinations, retain backups during replacement, and restore originals if a replacement fails.
A complete file replacement and an all-or-nothing batch are separate requirements, so per-file replacement alone does not satisfy the batch contract.
Temporary files and backups add disk work, but they provide a recovery path for failures during replacement.

This records the intended error-handling contract for in-place batches.
It does not establish crash recovery or a simultaneous filesystem transaction, and it does not certify that every rollback failure is handled.
The current writer performs rollback on a best-effort basis.

Sources: [Core specification, section 2.4](../../openspec/changes/archive/2026-05-09-smt-core/spec.md),
[core design, section 6](../../openspec/changes/archive/2026-05-09-smt-core/design.md).
Implementation reference: [batch writer](../../src/writer.rs).
