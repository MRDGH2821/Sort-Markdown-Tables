---
layout: default
title: Sort Markdown Tables
---

# Sort Markdown Tables

A zero-dependency, ultra-fast Rust CLI tool to keep your markdown tables automatically sorted and cleanly formatted.

> [!TIP]
> **Opt-In Safety**: Tables are sorted **only** when marked with an `<!-- smt -->` HTML comment directly above them. All other tables in your documents are left completely untouched!

---

## Quick Example

### Before Running `smt`

```markdown
<!-- smt type=numeric column=2 order=desc -->

| Student | Score | Grade |
| ------- | ----- | ----- |
| Bob     | 78    | C     |
| Alice   | 95    | A     |
| Charlie | 88    | B     |
```

### After Running `smt -i document.md`

```markdown
<!-- smt type=numeric column=2 order=desc -->

| Student | Score | Grade |
| ------- | ----- | ----- |
| Alice   | 95    | A     |
| Charlie | 88    | B     |
| Bob     | 78    | C     |
```

---

## Key Features

- 🎯 **Opt-in Control** — Only sort tables you explicitly choose to mark with `<!-- smt -->`.
- ⚡ **Flexible Sort Modes** — Sort numerically or lexicographically, by any column, ascending or descending, case-sensitive or insensitive.
- 🛡️ **Atomic & Safe Writes** — Two-phase execution pipeline guarantees zero file corruption if an error occurs.
- 🔍 **Check Mode for CI** — Easily verify whether markdown tables are sorted without modifying files.
- 📁 **Recursion & Globs** — Batch process entire documentation directories with `-r` or patterns like `docs/**/*.md`.

---

## Quick Start

### 1. Install `smt`

```bash
cargo install --git https://github.com/MRDGH2821/Sort-Markdown-Tables
```

### 2. Run Commands

```bash
# Sort a markdown file in-place
smt -i README.md

# Verify if files are sorted (ideal for CI pipelines)
smt --check docs/**/*.md

# Recursively scan and sort a directory
smt -r -i docs/
```

---

## Explore the Documentation

- 📦 [**Installation**](installation.html) — Learn how to install via Cargo, Nix Flakes, or pre-built binaries.
- 📖 [**Usage Guide**](usage.html) — Detailed guide on comment syntax, sort attributes, CLI flags, and exit codes.
- 🔌 [**Integrations**](integrations.html) — Set up `smt` with `pre-commit`, `git-hooks.nix`, `treefmt`, or GitHub Actions.
