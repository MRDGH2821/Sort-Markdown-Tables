# Integrations

## Pre-commit hook

Add to your `.pre-commit-config.yaml`:

```yaml
repos:
  - repo: https://github.com/MRDGH2821/Sort-Markdown-Tables
    rev: v0.3.1 # use the latest release tag
    hooks:
      - id: sort-markdown-tables # auto-fixes with smt -i
```

A check-only variant is also available:

```yaml
hooks:
  - id: sort-markdown-tables-check # fails if tables are unsorted
```

## Nix: git-hooks.nix module

For [cachix/git-hooks.nix](https://github.com/cachix/git-hooks.nix) users, import the provided module:

```nix
# flake.nix
{
  inputs.smt = {
    url = "github:MRDGH2821/Sort-Markdown-Tables";
    inputs.nixpkgs.follows = "nixpkgs";
  };
}
```

Then in your git-hooks configuration:

```nix
{
  imports = [ inputs.smt.gitHooksModules.default ];
  hooks.sort-markdown-tables.enable = true;
  # Optionally override the package:
  # hooks.sort-markdown-tables.package = pkgs.sort-markdown-tables;
}
```

## Nix: treefmt module

For [numtide/treefmt-nix](https://github.com/numtide/treefmt-nix) users:

```nix
{
  imports = [ inputs.smt.treefmtModules.default ];
  programs.sort-markdown-tables = {
    enable = true;
    # excludes = [ "vendor/**" ];
    # includes = [ "*.md" ];  # default
    # priority = 2;           # default
  };
}
```

## CI/CD validation

```bash
#!/bin/bash
smt --check "docs/**/*.md"
if [ $? -eq 1 ]; then
  echo "Markdown tables are unsorted. Run: smt -i 'docs/**/*.md'"
  exit 1
fi
```

## Bulk formatting

```bash
# Sort all markdown files in a directory
smt -i "**/*.md"
```
