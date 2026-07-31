# Design Specification: treefmt-nix Module Integration for Sort-Markdown-Tables

**Date**: 2026-07-31  
**Status**: Approved  
**Topic**: Exporting a reusable `treefmt-nix` module (`treefmtModules.default` / `treefmtModules.sort-markdown-tables`) for `sort-markdown-tables` and updating local `treefmt` evaluation to consume it.

---

## 1. Overview & Context

`Sort-Markdown-Tables` provides a CLI tool (`smt`) for sorting markdown tables. Downstream Nix flake projects using `treefmt-nix` for project formatting need a clean, standard module to enable `sort-markdown-tables` without manually writing `settings.formatter` definitions.

This specification adds:

1. A `treefmt-nix` program module in `nix/treefmt-module.nix` exposing `programs.sort-markdown-tables`.
2. Flake output exports for `treefmtModules.sort-markdown-tables` and `treefmtModules.default` in `flake.nix`.
3. Local configuration updates in `nix/formatter.nix` and `nix/treefmt.nix` to consume `programs.sort-markdown-tables`.

---

## 2. Module Specification (`nix/treefmt-module.nix`)

The module registers the option schema under `programs.sort-markdown-tables`:

- `programs.sort-markdown-tables.enable` (`bool`): Defaults to `false`. Enable formatting markdown tables using `smt`.
- `programs.sort-markdown-tables.package` (`package`): Defaults to `pkgs.sort-markdown-tables` (or fallback). The `smt` binary package.
- `programs.sort-markdown-tables.includes` (`listOf str`): Defaults to `["*.md"]`. Globs to include.
- `programs.sort-markdown-tables.excludes` (`listOf str`): Defaults to `[]`. Globs to exclude.
- `programs.sort-markdown-tables.priority` (`nullOr int`): Defaults to `2`. Formatter execution priority.

When `cfg.enable` is `true`, it outputs to `settings.formatter.sort-markdown-tables`:

```nix
settings.formatter.sort-markdown-tables = {
  command = lib.getExe cfg.package;
  includes = cfg.includes;
  excludes = cfg.excludes;
  options = ["-i"];
} // (lib.optionalAttrs (cfg.priority != null) { inherit (cfg) priority; });
```

---

## 3. Flake Output Integration (`flake.nix`)

Update `flake.nix` to combine `blueprint` outputs with `treefmtModules`:

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

This permits downstream flakes to consume the module:

```nix
{
  inputs = {
    treefmt-nix.url = "github:numtide/treefmt-nix";
    sort-markdown-tables.url = "github:MRDGH2821/Sort-Markdown-Tables";
  };
  # ...
  # imports = [ inputs.sort-markdown-tables.treefmtModules.default ];
  # programs.sort-markdown-tables.enable = true;
}
```

---

## 4. Local Evaluation Updates

### `nix/formatter.nix`

Include `./treefmt-module.nix` in the `imports` array of `inputs.treefmt.lib.evalModule pkgs`.

### `nix/treefmt.nix`

Replace the inline `settings.formatter.sort-markdown-tables` definition with:

```nix
programs.sort-markdown-tables = {
  enable = true;
  package = pkgs.callPackage ./packages/Sort-Markdown-Tables.nix { inherit inputs; };
};
```

---

## 5. Verification Plan

1. Evaluate `nix flake check` to verify all flake checks pass.
2. Run `nix fmt` to verify `sort-markdown-tables` executes correctly via `programs.sort-markdown-tables`.
3. Check `nix eval .#treefmtModules.default` to confirm the module export exists.
