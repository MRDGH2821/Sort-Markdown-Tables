# Design Specification: Nix Package & App via Numtide Blueprint & Crane

**Date**: 2026-07-26  
**Status**: Approved  
**Topic**: Adding Nix package derivation, app binary runnable, and cargo flake checks to `Sort-Markdown-Tables` via `numtide/blueprint` and `crane`.

---

## 1. Overview & Context

`Sort-Markdown-Tables` is a Rust CLI tool (`smt`) for sorting markdown tables. The repository uses `numtide/blueprint` for managing Nix flake outputs under the `nix/` directory.

This specification adds:

1. `crane` integration as a flake input for building Cargo projects with shared artifact layer caching.
2. A Nix package output named `Sort-Markdown-Tables` (and aliased to `default`).
3. A Nix app output exposing the binary `smt`.
4. Automated `cargo test` and `cargo clippy` checks under `nix/checks/`.

---

## 2. Flake Architecture & Inputs

### `flake.nix`

Update `flake.nix` to declare `crane`:

```nix
{
  description = "Sort Markdown Tables dev shell and Nix package";
  inputs = {
    blueprint = {
      inputs.nixpkgs.follows = "nixpkgs";
      url = "github:numtide/blueprint";
    };
    crane = {
      inputs.nixpkgs.follows = "nixpkgs";
      url = "github:ipetkov/crane";
    };
    git-hooks = {
      inputs.nixpkgs.follows = "nixpkgs";
      url = "github:cachix/git-hooks.nix";
    };
    llm-agents = {
      inputs.nixpkgs.follows = "nixpkgs";
      url = "github:numtide/llm-agents.nix";
    };
    nixpkgs.url = "github:NixOS/nixpkgs?ref=nixos-unstable";
    pedantix = {
      inputs.nixpkgs.follows = "nixpkgs";
      url = "github:swarsel/pedantix";
    };
    treefmt = {
      inputs.nixpkgs.follows = "nixpkgs";
      url = "github:numtide/treefmt-nix";
    };
  };
  nixConfig = {
    extra-substituters = ["https://cache.numtide.com"];
    extra-trusted-public-keys = ["niks3.numtide.com-1:DTx8wZduET09hRmMtKdQDxNNthLQETkc/yaX7M4qK0g="];
  };
  outputs = inputs:
    inputs.blueprint {
      inherit inputs;
      prefix = "nix/";
    };
}
```

---

## 3. Package & App Definitions

### `nix/packages/Sort-Markdown-Tables.nix`

Defines the `Sort-Markdown-Tables` package derivation using `craneLib.buildPackage`:

- `pname`: `"Sort-Markdown-Tables"`
- `version`: Parsed dynamically from `Cargo.toml`
- Source clean filter: `craneLib.cleanCargoSource`
- Precompiled `cargoArtifacts` built via `craneLib.buildDepsOnly`

### `nix/packages/default.nix`

Re-exports `perSystem.self.Sort-Markdown-Tables` to make `nix build` construct `packages.${system}.default`.

### `nix/apps/default.nix`

Uses `inputs.blueprint.lib.mkApp` to wrap `perSystem.self.Sort-Markdown-Tables` with binary executable `smt`.

---

## 4. Flake Checks

### `nix/checks/cargo-test.nix`

Runs unit and integration tests via `craneLib.cargoTest` using the shared `cargoArtifacts`.

### `nix/checks/cargo-clippy.nix`

Runs linter checks via `craneLib.cargoClippy` with `--all-targets -- -D warnings`.

---

## 5. Verification Plan

1. **`nix build`**: Verify build completion and check `./result/bin/smt --version`.
2. **`nix build .#Sort-Markdown-Tables`**: Verify explicit package attribute build.
3. **`nix run . -- --help`**: Verify app binary execution.
4. **`nix flake check`**: Verify pre-commit, treefmt, cargo-test, and cargo-clippy all pass.
5. **`nix fmt`**: Format all Nix files with treefmt.
