# Docs/src Index Page & Workflow Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add a dedicated index page in `docs/src/index.md`, set up the docs site configuration and deployment workflow, clean up stale git index entries, and commit the changes on a new feature branch.

**Architecture:** Organize documentation files under `docs/src/` for Zensical/MkDocs. Clean up stale tracked paths in the root `docs/` folder, configure `zensical.toml` and `.github/workflows/docs.yml` appropriately, and verify everything with `nix fmt` and test suites.

**Tech Stack:** Markdown, Zensical / MkDocs, GitHub Actions, Nix (`nix fmt`), Git.

## Global Constraints

- Follow Conventional Commits format with appropriate scopes (`docs`, `ci`, etc.).
- Maintain AI model signoff trailers (`Co-authored-by` and `AI-Model` git trailer).
- All changes must pass `nix fmt` formatting and `pre-commit` / `prek` hooks.

---

### Task 1: Create feature branch and clean up stale git index entries

**Files:**

- Modify (Git Index): `docs/architecture.md`, `docs/installation.md`, `docs/integrations.md`, `docs/usage.md` (remove deleted paths from git cache)

**Interfaces:**

- Consumes: Current `main` branch state
- Produces: Clean git working tree on new branch `docs/index-page`

- [ ] **Step 1: Create and checkout new branch**

```bash
git checkout -b docs/index-page
```

- [ ] **Step 2: Remove old root docs files from git index**

```bash
git rm --cached docs/architecture.md docs/installation.md docs/integrations.md docs/usage.md
```

- [ ] **Step 3: Verify git status**

```bash
git status
```

---

### Task 2: Create `docs/src/index.md` and configure documentation artifacts

**Files:**

- Create: `docs/src/index.md`
- Track/Verify: `zensical.toml`, `.github/workflows/docs.yml`

**Interfaces:**

- Consumes: README documentation summary and Zensical configuration
- Produces: Validated `docs/src/index.md` landing page for MkDocs/Zensical

- [ ] **Step 1: Write `docs/src/index.md`**

````markdown
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
````

## Licence

See [LICENCE.txt](https://github.com/MRDGH2821/Sort-Markdown-Tables/blob/main/LICENCE.txt).

## Changelog

See [CHANGELOG.md](https://github.com/MRDGH2821/Sort-Markdown-Tables/blob/main/CHANGELOG.md).

````

- [ ] **Step 2: Format files with `nix fmt`**

Run: `nix fmt`
Expected: Clean formatting without errors.

---

### Task 3: Commit changes on the new branch

**Files:**
- Commit: `docs/src/index.md`, `zensical.toml`, `.github/workflows/docs.yml`, `.gitignore`, `nix/treefmt.nix`, `nix/devshell.nix`

- [ ] **Step 1: Stage all relevant documentation and configuration changes**

```bash
git add docs/src/ zensical.toml .github/workflows/docs.yml .gitignore nix/treefmt.nix
````

- [ ] **Step 2: Commit changes**

```bash
git commit -m "docs(smt): add index page in docs/src and setup zensical workflow

- Created docs/src/index.md landing page
- Configured zensical.toml and docs deployment workflow
- Cleaned up stale root docs paths from git index
- Added site/ to .gitignore and treefmt global excludes

Co-authored-by: Gemini 3.6 Flash via Antigravity <noreply@google.com>" --trailer="AI-Model: gemini-3.6-flash"
```

- [ ] **Step 3: Verify git log and working tree clean status**

```bash
git status && git log -n 1
```
