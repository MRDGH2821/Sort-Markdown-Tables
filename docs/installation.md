---
layout: default
title: Installation Guide
---

## Installation Guide

`Sort Markdown Tables` (`smt`) is available via standard Rust tools, Nix Flakes, or pre-built release binaries.

---

### Installation Methods

#### Cargo (From Source)

Requirements: **Rust 1.70+**

```bash
cargo install --git https://github.com/MRDGH2821/Sort-Markdown-Tables
```

Verify your installation:

```bash
smt --version
```

#### Nix Flakes

Run directly without permanent installation:

```bash
nix run github:MRDGH2821/Sort-Markdown-Tables -- --help
```

Or add `smt` as an input in your `flake.nix`:

```nix
inputs.smt = {
  url = "github:MRDGH2821/Sort-Markdown-Tables";
  inputs.nixpkgs.follows = "nixpkgs";
};
```

Then include `inputs.smt.packages.${system}.default` in your `environment.systemPackages` or `devShells`.

#### Pre-built Binaries

Download pre-compiled binaries for Linux, macOS, or Windows directly from our Github Releases:

1. Visit the [GitHub Releases Page](https://github.com/MRDGH2821/Sort-Markdown-Tables/releases).
2. Download the binary for your architecture (e.g., `smt-x86_64-unknown-linux-gnu`, or a Windows `.exe`).
3. Rename it to `smt` (`smt.exe` on Windows), place it in your `$PATH`, and make it executable on Unix.

#### Verify Signed Downloads

Releases built with the Packslip-enabled workflow include `packslip.sigstore.json`.
Older releases without that asset cannot use this verification method.
Install the [Packslip CLI](https://packslip.dev/docs/getting-started/#install-packslip)
and use GitHub CLI to download a manifest and binary from the same release:

```sh
REPO=MRDGH2821/Sort-Markdown-Tables
TAG=$(gh release view --repo "$REPO" --json tagName --jq .tagName)
gh release download "$TAG" --repo "$REPO" \
  --pattern packslip.sigstore.json --pattern smt-x86_64-unknown-linux-gnu
packslip verify packslip.sigstore.json \
  --identity "https://github.com/$REPO/.github/workflows/release.yml@refs/tags/$TAG" \
  --issuer https://token.actions.githubusercontent.com \
  --artifact smt-x86_64-unknown-linux-gnu
```

Use the binary name for your platform. Verification checks the workflow identity,
signature, and binary digest. The manifest exposes each raw binary as the `smt`
command; Windows GNU and GNULLVM builds use the `gnu` and `gnullvm` variants,
and ARMv7 soft-float builds use `soft-float`. MSVC and ARMv7 hard-float are defaults.

The manifest also links GitHub build attestations. Check those separately:

```sh
gh attestation verify smt-x86_64-unknown-linux-gnu --repo "$REPO"
```

---

### EditorConfig Line Endings

When writing, `smt` uses the effective EditorConfig `end_of_line` setting for
the path being written. `lf`, `crlf`, and `cr` override the input; when the
setting is absent or `unset`, each input line's terminator is preserved.
In-place writes and file input sent to stdout use the input path; `--write`
uses its destination path. Stdin sent directly to stdout skips config lookup
and preserves input terminators, with the existing final-newline behavior for
unterminated input.

### Troubleshooting

> [!NOTE]
> **PATH Configuration**: If running `smt` in your terminal outputs `command not found`, ensure `~/.cargo/bin` (for Cargo) or `~/.local/bin` (for manually downloaded binaries) is in your shell's `PATH` environment variable:
>
> ```bash
> export PATH="$HOME/.cargo/bin:$PATH"
> ```
