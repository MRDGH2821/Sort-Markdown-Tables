# Nix Package & App Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add Nix package derivation (`Sort-Markdown-Tables`), app binary runnable (`smt`), and cargo flake checks (`cargo test` & `cargo clippy`) to `flake.nix` using `numtide/blueprint` and `crane`.

**Architecture:** Update `flake.nix` to include `crane` input, create `nix/packages/Sort-Markdown-Tables.nix` and `nix/packages/default.nix` for building the binary, create `nix/apps/default.nix` to expose `nix run .`, and add `nix/checks/cargo-test.nix` & `nix/checks/cargo-clippy.nix` for flake checks.

**Tech Stack:** Nix (Flakes, numtide/blueprint, crane), Rust.

## Global Constraints

- Must use `numtide/blueprint` layout conventions under `nix/`.
- Package name must be `Sort-Markdown-Tables` (re-exported to `default`).
- Binary executable name must be `smt`.
- Must pass `nix flake check` and `nix fmt`.
- All commits must follow conventional commits (`feat(nix)`, `chore(nix)`, etc.), include `Co-authored-by` and `AI-Model` trailers, and pass pre-commit hooks.

---

### Task 1: Add `crane` Input to `flake.nix`

**Files:**

- Modify: `flake.nix`

**Interfaces:**

- Consumes: None
- Produces: `inputs.crane` available to all `nix/` modules processed by `numtide/blueprint`

- [ ] **Step 1: Update `flake.nix` to add `crane` input**

Edit `flake.nix` to include:

```nix
    crane = {
      inputs.nixpkgs.follows = "nixpkgs";
      url = "github:ipetkov/crane";
    };
```

- [ ] **Step 2: Update `flake.lock` with `nix flake lock` or `nix flake check`**

Run:

```bash
nix flake lock
```

Expected: `flake.lock` updated with `crane` input entry.

- [ ] **Step 3: Commit `flake.nix` and `flake.lock`**

Run:

```bash
git add flake.nix flake.lock
git commit --trailer="Co-authored-by: Gemini 3.6 Flash via Antigravity CLI <noreply@google.com>" --trailer="AI-Model: gemini-3.6-flash" -m "feat(nix): add crane input to flake.nix"
```

---

### Task 2: Create Package Derivations (`Sort-Markdown-Tables` and `default`)

**Files:**

- Create: `nix/packages/Sort-Markdown-Tables.nix`
- Create: `nix/packages/default.nix`

**Interfaces:**

- Consumes: `inputs.crane`, `pkgs`, `Cargo.toml`
- Produces: `packages.${system}.Sort-Markdown-Tables` and `packages.${system}.default`

- [ ] **Step 1: Create `nix/packages/Sort-Markdown-Tables.nix`**

Create `nix/packages/Sort-Markdown-Tables.nix` with the following content:

```nix
{
  inputs,
  pkgs,
  ...
}: let
  craneLib = inputs.crane.mkLib pkgs;
  src = craneLib.cleanCargoSource (craneLib.path ../..);
  commonArgs = {
    inherit src;
    pname = "Sort-Markdown-Tables";
    version = (builtins.fromTOML (builtins.readFile ../../Cargo.toml)).package.version;
    strictDeps = true;
  };
  cargoArtifacts = craneLib.buildDepsOnly commonArgs;
in
  craneLib.buildPackage (commonArgs // {
    inherit cargoArtifacts;
  })
```

- [ ] **Step 2: Create `nix/packages/default.nix`**

Create `nix/packages/default.nix` with the following content:

```nix
{ perSystem, ... }:
perSystem.self.Sort-Markdown-Tables
```

- [ ] **Step 3: Verify package build with `nix build`**

Run:

```bash
nix build
./result/bin/smt --version
```

Expected: Output showing `smt 0.1.4`.

- [ ] **Step 4: Commit package derivations**

Run:

```bash
git add nix/packages/Sort-Markdown-Tables.nix nix/packages/default.nix
git commit --trailer="Co-authored-by: Gemini 3.6 Flash via Antigravity CLI <noreply@google.com>" --trailer="AI-Model: gemini-3.6-flash" -m "feat(nix): add Sort-Markdown-Tables package derivation"
```

---

### Task 3: Create App Binary Runnable (`nix/apps/default.nix`)

**Files:**

- Create: `nix/apps/default.nix`

**Interfaces:**

- Consumes: `inputs.blueprint`, `perSystem.self.Sort-Markdown-Tables`
- Produces: `apps.${system}.default`

- [ ] **Step 1: Create `nix/apps/default.nix`**

Create `nix/apps/default.nix` with the following content:

```nix
{
  inputs,
  perSystem,
  ...
}:
inputs.blueprint.lib.mkApp {
  drv = perSystem.self.Sort-Markdown-Tables;
  name = "smt";
}
```

- [ ] **Step 2: Verify app execution with `nix run . -- --help`**

Run:

```bash
nix run . -- --help
```

Expected: Displays `smt` CLI help options.

- [ ] **Step 3: Commit app definition**

Run:

```bash
git add nix/apps/default.nix
git commit --trailer="Co-authored-by: Gemini 3.6 Flash via Antigravity CLI <noreply@google.com>" --trailer="AI-Model: gemini-3.6-flash" -m "feat(nix): add smt binary app default output"
```

---

### Task 4: Add Crane Flake Checks (`cargo-test` and `cargo-clippy`)

**Files:**

- Create: `nix/checks/cargo-test.nix`
- Create: `nix/checks/cargo-clippy.nix`

**Interfaces:**

- Consumes: `inputs.crane`, `pkgs`, `Cargo.toml`
- Produces: `checks.${system}.cargo-test` and `checks.${system}.cargo-clippy`

- [ ] **Step 1: Create `nix/checks/cargo-test.nix`**

Create `nix/checks/cargo-test.nix` with the following content:

```nix
{
  inputs,
  pkgs,
  ...
}: let
  craneLib = inputs.crane.mkLib pkgs;
  src = craneLib.cleanCargoSource (craneLib.path ../..);
  commonArgs = {
    inherit src;
    pname = "Sort-Markdown-Tables";
    version = (builtins.fromTOML (builtins.readFile ../../Cargo.toml)).package.version;
    strictDeps = true;
  };
  cargoArtifacts = craneLib.buildDepsOnly commonArgs;
in
  craneLib.cargoTest (commonArgs // {
    inherit cargoArtifacts;
  })
```

- [ ] **Step 2: Create `nix/checks/cargo-clippy.nix`**

Create `nix/checks/cargo-clippy.nix` with the following content:

```nix
{
  inputs,
  pkgs,
  ...
}: let
  craneLib = inputs.crane.mkLib pkgs;
  src = craneLib.cleanCargoSource (craneLib.path ../..);
  commonArgs = {
    inherit src;
    pname = "Sort-Markdown-Tables";
    version = (builtins.fromTOML (builtins.readFile ../../Cargo.toml)).package.version;
    strictDeps = true;
  };
  cargoArtifacts = craneLib.buildDepsOnly commonArgs;
in
  craneLib.cargoClippy (commonArgs // {
    inherit cargoArtifacts;
    cargoClippyExtraArgs = "--all-targets -- -D warnings";
  })
```

- [ ] **Step 3: Verify flake checks with `nix flake check`**

Run:

```bash
nix flake check
```

Expected: `cargo-test`, `cargo-clippy`, `pre-commit-check`, and `formatting` checks build and pass.

- [ ] **Step 4: Commit flake checks**

Run:

```bash
git add nix/checks/cargo-test.nix nix/checks/cargo-clippy.nix
git commit --trailer="Co-authored-by: Gemini 3.6 Flash via Antigravity CLI <noreply@google.com>" --trailer="AI-Model: gemini-3.6-flash" -m "feat(nix): add cargo test and clippy flake checks"
```

---

### Task 5: Final Formatting and Verification

**Files:**

- Modify: any unformatted nix files

- [ ] **Step 1: Format codebase with `nix fmt`**

Run:

```bash
nix fmt
```

- [ ] **Step 2: Run complete validation suite**

Run:

```bash
nix flake check && nix build .#Sort-Markdown-Tables && ./result/bin/smt --version
```

Expected: All checks pass, package builds, binary executes cleanly.

- [ ] **Step 3: Final commit if any formatting changes**

Run:

```bash
git add .
git status --porcelain
# Commit only if git status is not empty
```
