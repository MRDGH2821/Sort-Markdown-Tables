---
status: accepted
---

# Preserve stable order within one sort column

Equal comparison values must produce predictable output without rearranging unrelated row content.
Use a stable sort on one selected column so that rows with equal sort keys retain their original relative order.
This retains author-controlled order for ties and leaves additional sort columns outside the core scope.

Sorting and check mode must use the same comparison rules.
Numeric and text comparison, case sensitivity, and direction determine whether keys are equal.
The source documents disagree about numeric fallback details, which remain recorded in the [decision index](README.md#source-differences).

Sources: [Core specification, sections 2.2, 3.4, and 6](../../openspec/changes/archive/2026-05-09-smt-core/spec.md),
[core design, section 5](../../openspec/changes/archive/2026-05-09-smt-core/design.md).
