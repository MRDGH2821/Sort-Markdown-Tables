# Integrations Guide

Integrate `smt` into your development workflow, git hooks, formatters, and CI/CD pipelines.

---

## 1. Pre-commit Framework

Add `smt` to your `.pre-commit-config.yaml`:

=== "Auto-Fix Mode (`smt -i`)"

    Automatically formats and sorts tables before every commit:

    ```yaml
    repos:
      - repo: https://github.com/MRDGH2821/Sort-Markdown-Tables
        rev: v0.4.0 # Use latest release tag
        hooks:
          - id: sort-markdown-tables
    ```

=== "Check-Only Mode (`smt --check`)"

    Prevents committing unsorted tables without modifying files:

    ```yaml
    repos:
      - repo: https://github.com/MRDGH2821/Sort-Markdown-Tables
        rev: v0.4.0 # Use latest release tag
        hooks:
          - id: sort-markdown-tables-check
    ```

---

## 2. Nix Flakes Integration

=== "git-hooks.nix"

    For [cachix/git-hooks.nix](https://github.com/cachix/git-hooks.nix) users, import the exported module in your `flake.nix`:

    ```nix
    {
      inputs.smt = {
        url = "github:MRDGH2821/Sort-Markdown-Tables";
        inputs.nixpkgs.follows = "nixpkgs";
      };
    }
    ```

    Then enable `sort-markdown-tables` in your git-hooks config:

    ```nix
    {
      imports = [ inputs.smt.gitHooksModules.default ];
      hooks.sort-markdown-tables.enable = true;
    }
    ```

=== "treefmt-nix"

    For [numtide/treefmt-nix](https://github.com/numtide/treefmt-nix) users, import the treefmt module:

    ```nix
    {
      imports = [ inputs.smt.treefmtModules.default ];
      programs.sort-markdown-tables = {
        enable = true;
        # includes = [ "*.md" ];  # Default
        # excludes = [ "vendor/**" ];
      };
    }
    ```

---

## 3. GitHub Actions CI/CD Pipeline

Enforce sorted markdown tables on every pull request or push with GitHub Actions:

```yaml
name: Verify Markdown Table Sorting

on:
  push:
    branches: [main]
  pull_request:
    branches: [main]

jobs:
  check-tables:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Install Rust
        uses: dtolnay/rust-toolchain@stable

      - name: Install smt
        run: cargo install --git https://github.com/MRDGH2821/Sort-Markdown-Tables

      - name: Check markdown tables
        run: smt --check -r docs/ README.md
```

---

## 4. Bulk Repository Formatting

Run `smt` across all markdown files in your project with a single command:

```bash
# Recursively format all markdown files in the repository
smt -r -i ./
```
