# Architecture decision records

These records capture consequential choices from `openspec/` and `docs/superpowers/`.
They were reconstructed on 2026-10-13 and cross-checked against this checkout.
An accepted record describes the design choice, not a claim that the implementation satisfies every requirement.

Use [GLOSSARY.md](../../GLOSSARY.md) for domain terms.
Read the relevant decision before changing table selection, row comparison, batch writes, or line-ending handling.
The linked source documents retain the detailed requirements and historical plans.

## Recorded decisions

| Record                                           | Choice                                                   |
| ------------------------------------------------ | -------------------------------------------------------- |
| [0001](0001-opt-in-sorting-and-raw-markdown.md)  | Opt-in tables and raw Markdown preservation              |
| [0002](0002-stable-single-column-row-order.md)   | Stable ordering by one sort column                       |
| [0003](0003-prepare-before-batch-replacement.md) | Prepare inputs before replacing an in-place batch        |
| [0004](0004-editorconfig-line-endings.md)        | Output-path EditorConfig policy and per-line terminators |

## Source differences

The sources contain disagreements that these records do not resolve.
Current code provides evidence of behavior, rather than retroactive approval of every difference.
Keep behavior changes separate from this documentation reconstruction.

| Topic               | Source rule or proposal                                                                                                          | Current evidence                                                                                      |
| ------------------- | -------------------------------------------------------------------------------------------------------------------------------- | ----------------------------------------------------------------------------------------------------- |
| Marker spacing      | The core specification forbids blank lines after a marker, but its examples include them.                                        | The parser accepts and retains blank lines before the header.                                         |
| Numeric fallback    | The core specification preserves the order of non-numeric cells, while the design's comparator and task 4.1 use text comparison. | The sorter compares non-numeric cells as text and reverses the whole comparison for descending order. |
| Scientific notation | The original project plan treats `1e3` as non-numeric.                                                                           | The sorter uses Rust floating-point parsing, which accepts `1e3` as a number.                         |
| Final newline       | The baseline architecture calls for preserving the final newline. The EditorConfig design retains existing behavior.             | File output omits the last terminator. Stdout adds a final terminator.                                |
| Version interface   | The version implementation plan adds `smt version`, while its design specifies only `--version` and `-V`.                        | The CLI exposes the two flags and treats `version` as a positional input.                             |
| Rust task aliases   | The EditorConfig design requests mise tasks for tests and Clippy.                                                                | `mise.toml` includes Rust and runs Cargo tests in `check`, but has no `test` or `clippy` task.        |

Evidence: [parser](../../src/parser.rs), [sorter](../../src/sorter.rs), [writer](../../src/writer.rs),
[CLI](../../src/cli.rs), [integration tests](../../tests/integration_test.rs), and [mise configuration](../../mise.toml).
Original core sources: [project plan](../../openspec/specs/smt/plan.md), [architecture](../../openspec/specs/smt/architecture.md),
[archived specification](../../openspec/changes/archive/2026-05-09-smt-core/spec.md),
[design](../../openspec/changes/archive/2026-05-09-smt-core/design.md), and [tasks](../../openspec/changes/archive/2026-05-09-smt-core/tasks.md).

## Historical plans

The Nix designs are labeled approved in their source documents.
This checkout contains no `flake.nix` or `nix/` implementation, so those designs are historical references rather than available interfaces.
The documentation-site plan also differs from the current Jekyll workflow, which builds from `docs/`.

| Area                    | Recorded intent                                                                                             | Sources                                                                                                                                     |
| ----------------------- | ----------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------- |
| Nix package and app     | Use Blueprint for flake layout and Crane for Cargo builds, dependency artifacts, and checks.                | [Design](../superpowers/specs/2026-07-26-nix-package-blueprint-design.md), [plan](../superpowers/plans/2026-07-26-nix-package-blueprint.md) |
| treefmt-nix integration | Export a reusable program module that invokes `smt -i`, with downstream package and file-selection options. | [Design](../superpowers/specs/2026-07-31-treefmt-nix-module-design.md), [plan](../superpowers/plans/2026-07-31-treefmt-nix-module.md)       |
| Version display         | Derive the displayed version from Cargo metadata. The plan's extra subcommand is not implemented.           | [Design](../superpowers/specs/2026-07-26-version-display-design.md), [plan](../superpowers/plans/2026-07-26-version-display.md)             |
| Documentation site      | Move site content under `docs/src/` and use Zensical or MkDocs. That layout is absent from this checkout.   | [Plan](../superpowers/plans/2026-08-04-docs-src-index.md), [current workflow](../../.github/workflows/docs.yml)                             |

The [core proposal](../../openspec/changes/archive/2026-05-09-smt-core/proposal.md) records the initial scope.
The [OpenSpec configuration](../../openspec/config.yaml) records the workflow rules for those artifacts.
Task checkboxes describe historical work and do not prove that every listed file, dependency, or interface exists today.
