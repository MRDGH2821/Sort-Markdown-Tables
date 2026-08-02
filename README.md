# Sort Markdown Tables

[![Copier](https://img.shields.io/endpoint?url=https://raw.githubusercontent.com/copier-org/copier/refs/heads/master/img/badge/black-badge.json)](https://github.com/copier-org/copier)

A zero-dependency Rust CLI tool to sort markdown tables. Tables are opted-in via `<!-- smt -->` HTML comments.

## Features

- **Opt-in sorting** — Only tables marked with `<!-- smt -->` are sorted
- **Multiple sort modes** — Numeric, lexicographic (case-sensitive/insensitive), ascending/descending
- **Atomic writes** — Two-phase pipeline ensures zero file modifications on any error
- **Glob support** — Process multiple files with patterns like `docs/**/*.md`
- **Recursive scan** — Scan directories automatically with `-r`
- **Check mode** — Validate if files are sorted without modifying them
- **In-place editing** — Modify files directly or output to stdout

## Quick Start

```bash
# Install
cargo install --git https://github.com/MRDGH2821/Sort-Markdown-Tables

# Sort in-place
smt -i README.md

# Check (CI-friendly)
smt --check README.md
```

## Documentation

- [Installation](docs/installation.md) — cargo, pre-built binaries, Nix Flakes
- [Usage](docs/usage.md) — CLI flags, table format, sort options, exit codes
- [Integrations](docs/integrations.md) — pre-commit, git-hooks.nix, treefmt, CI/CD
- [Architecture](docs/architecture.md) — pipeline, algorithm, module layout, dependencies

## Licence

See [LICENCE.txt](LICENCE.txt).

## Changelog

See [CHANGELOG.md](CHANGELOG.md).
