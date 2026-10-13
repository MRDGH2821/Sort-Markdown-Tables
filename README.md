# Sort Markdown Tables

[![Copier](https://img.shields.io/endpoint?url=https://raw.githubusercontent.com/copier-org/copier/refs/heads/master/img/badge/black-badge.json)](https://github.com/copier-org/copier)

A Rust CLI tool to sort Markdown tables opted in with `<!-- smt -->` comments.

## Features

- **Opt-in sorting** — Only tables marked with `<!-- smt -->` are sorted.
- **Multiple sort modes** — Numeric or lexicographic; case-sensitive or insensitive; ascending or descending.
- **Safe in-place writes** — Files are updated atomically after all inputs are processed successfully.
- **Glob and recursive input** — Process patterns such as `docs/**/*.md` or recursively scan directories with `-r`.
- **Check mode** — Verify whether files are sorted without modifying them.
- **EditorConfig line endings** — Honor the effective `end_of_line` setting for the output path; preserve mixed input endings when the setting is absent or `unset`.

## Quick Start

```sh
# Install from source
cargo install --git https://github.com/MRDGH2821/Sort-Markdown-Tables

# Sort a file in place
smt -i README.md

# Write sorted output to another file
smt input.md --write output.md

# Check whether files are sorted
smt --check README.md
```

## Documentation

- [Installation and EditorConfig behavior](docs/installation.md)
- [Usage](docs/usage.md) — CLI flags, table format, sort options, and exit codes
- [Integrations](docs/integrations.md) — `pre-commit`, `git-hooks.nix`, `treefmt`, and GitHub Actions

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) for setup, checks, and the contribution workflow.

## Licence

See [LICENCE.txt](LICENCE.txt).

## Changelog

See [CHANGELOG.md](CHANGELOG.md).
