# treefmt-nix Module Integration Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Create and export a reusable `treefmt-nix` module (`treefmtModules.default` / `treefmtModules.sort-markdown-tables`) for `sort-markdown-tables`, and update the project's Nix configuration to consume it.

**Architecture:** A new Nix module file `nix/treefmt-module.nix` defines options under `programs.sort-markdown-tables` and configures `settings.formatter.sort-markdown-tables` when enabled. `flake.nix` exports `treefmtModules` in flake outputs. `nix/formatter.nix` and `nix/treefmt.nix` are updated to import and enable `programs.sort-markdown-tables`.

**Tech Stack:** Nix, `treefmt-nix`, `numtide/blueprint`, `git-hooks.nix`.

## Global Constraints

- Preserve all existing `nix/` structure under `numtide/blueprint`.
- Follow Conventional Commits format with valid scope from `cog.toml` (`treefmt` or `nix`).
- Ensure all commits include the required `Co-authored-by` AI trailer.

---

### Task 1: Create `nix/treefmt-module.nix`

**Files:**

- Create: `nix/treefmt-module.nix`

**Interfaces:**

- Consumes: `pkgs`, `config`, `lib` from `treefmt-nix` module evaluation.
- Produces: `options.programs.sort-markdown-tables` and `config.settings.formatter.sort-markdown-tables`.

- [ ] **Step 1: Create `nix/treefmt-module.nix`**

Write the treefmt-nix program module definition:

```nix
{
  config,
  lib,
  pkgs,
  ...
}: let
  cfg = config.programs.sort-markdown-tables;
in {
  options.programs.sort-markdown-tables = {
    enable = lib.mkEnableOption "sort-markdown-tables";

    package = lib.mkOption {
      type = lib.types.package;
      default = pkgs.sort-markdown-tables or pkgs.smt or (pkgs.callPackage ./packages/Sort-Markdown-Tables.nix {});
      defaultText = lib.literalExpression "pkgs.sort-markdown-tables";
      description = "The sort-markdown-tables package to use.";
    };

    includes = lib.mkOption {
      type = lib.types.listOf lib.types.str;
      default = ["*.md"];
      description = "List of files/globs to include for formatting.";
    };

    excludes = lib.mkOption {
      type = lib.types.listOf lib.types.str;
      default = [];
      description = "List of files/globs to exclude from formatting.";
    };

    priority = lib.mkOption {
      type = lib.types.nullOr lib.types.int;
      default = 2;
      description = "Formatter priority.";
    };
  };

  config = lib.mkIf cfg.enable {
    settings.formatter.sort-markdown-tables = {
      command = lib.getExe cfg.package;
      includes = cfg.includes;
      excludes = cfg.excludes;
      options = ["-i"];
    } // (lib.optionalAttrs (cfg.priority != null) { inherit (cfg) priority; });
  };
}
```

- [ ] **Step 2: Parse check with `nix-instantiate`**

Run: `nix-instantiate --parse nix/treefmt-module.nix`
Expected: AST output without parse errors.

- [ ] **Step 3: Commit `nix/treefmt-module.nix`**

```bash
git add nix/treefmt-module.nix
git commit --trailer="AI-Model: gemini-3.6-flash" -m "feat(treefmt): add sort-markdown-tables treefmt-nix program module

Co-authored-by: Gemini 3.6 Flash via Antigravity <noreply@google.com>"
```

---

### Task 2: Export `treefmtModules` in `flake.nix` and update `nix/formatter.nix` & `nix/treefmt.nix`

**Files:**

- Modify: `flake.nix`
- Modify: `nix/formatter.nix`
- Modify: `nix/treefmt.nix`

**Interfaces:**

- Consumes: `nix/treefmt-module.nix`
- Produces: `flake.outputs.treefmtModules.sort-markdown-tables` and `flake.outputs.treefmtModules.default`

- [ ] **Step 1: Update `flake.nix` to export `treefmtModules`**

Modify `flake.nix` outputs function:

```nix
  outputs = inputs:
    (inputs.blueprint {
      inherit inputs;
      prefix = "nix/";
    })
    // {
      treefmtModules = rec {
        sort-markdown-tables = ./nix/treefmt-module.nix;
        default = sort-markdown-tables;
      };
    };
```

- [ ] **Step 2: Update `nix/formatter.nix` to import `treefmt-module.nix`**

In `nix/formatter.nix`, update `treefmtEval` imports list:

```nix
  treefmtEval = inputs.treefmt.lib.evalModule pkgs {
    imports = [
      ./treefmt-module.nix
      ./treefmt.nix
      inputs.pedantix.treefmtModules.default
    ];
  };
```

- [ ] **Step 3: Update `nix/treefmt.nix` to use `programs.sort-markdown-tables`**

In `nix/treefmt.nix`:

1. Add `programs.sort-markdown-tables`:

```nix
    sort-markdown-tables = {
      enable = true;
      package = pkgs.callPackage ./packages/Sort-Markdown-Tables.nix { inherit inputs; };
      excludes = [
        "**/openspec/**"
        "openspec/**"
        "tests/fixtures/**"
      ];
    };
```

2. Remove the old `settings.formatter.sort-markdown-tables` block from `settings.formatter`.

- [ ] **Step 4: Verify module evaluation and formatting**

Run: `nix eval .#treefmtModules.default`
Expected: Output showing path to `./nix/treefmt-module.nix`.

Run: `rtk nix fmt -- --fail-on-change`
Expected: PASS (or format files without error).

- [ ] **Step 5: Commit changes**

```bash
git add flake.nix nix/formatter.nix nix/treefmt.nix
git commit --trailer="AI-Model: gemini-3.6-flash" -m "feat(treefmt): export treefmtModules and consume in local nix treefmt config

Co-authored-by: Gemini 3.6 Flash via Antigravity <noreply@google.com>"
```

---

### Task 3: Enable `treefmt` pre-commit hook and add `treefmt` package to `nix/devshell.nix`

**Files:**

- Modify: `nix/checks/pre-commit-check.nix`
- Modify: `nix/devshell.nix`

**Interfaces:**

- Consumes: `nix/formatter.nix`

- [ ] **Step 1: Enable `treefmt` in `nix/checks/pre-commit-check.nix`**

Add `treefmt` hook to `hooks`:

```nix
    treefmt = {
      enable = true;
      package = import ../formatter.nix { inherit inputs pkgs; };
    };
```

- [ ] **Step 2: Add treefmt wrapper to `nix/devshell.nix` packages**

In `nix/devshell.nix`, add `(import ./formatter.nix { inherit inputs pkgs; })` to `packages`:

```nix
    packages = [
      (import ./formatter.nix { inherit inputs pkgs; })
      cross-build-all
      cross-build-seq

      pkgs.nil
      pkgs.nixd
      pkgs.act
    ];
```

- [ ] **Step 3: Verify all flake checks**

Run: `rtk nix flake check`
Expected: All checks pass clean.

- [ ] **Step 4: Commit changes**

```bash
git add nix/checks/pre-commit-check.nix nix/devshell.nix .pre-commit-config.yaml
git commit --trailer="AI-Model: gemini-3.6-flash" -m "feat(nix): enable treefmt in pre-commit checks and devshell packages

Co-authored-by: Gemini 3.6 Flash via Antigravity <noreply@google.com>"
```
