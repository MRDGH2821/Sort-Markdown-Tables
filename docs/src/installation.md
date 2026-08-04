# Installation Guide

`Sort Markdown Tables` (`smt`) is available via standard Rust tools, Nix Flakes, or pre-built release binaries.

---

## Installation Methods

=== "Cargo (From Source)"

    Requirements: **Rust 1.70+**

    ```bash
    cargo install --git https://github.com/MRDGH2821/Sort-Markdown-Tables
    ```

    Verify your installation:

    ```bash
    smt --version
    ```

=== "Nix Flakes"

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

=== "Pre-built Binaries"

    Download pre-compiled binaries for Linux, macOS, or Windows directly from our Github Releases:

    1. Visit the [GitHub Releases Page](https://github.com/MRDGH2821/Sort-Markdown-Tables/releases).
    2. Download the archive for your architecture (e.g., `smt-x86_64-unknown-linux-gnu.tar.gz`).
    3. Extract the `smt` binary and move it to a directory in your `$PATH` (such as `/usr/local/bin` or `~/.local/bin`).

---

## Troubleshooting

> [!NOTE]
> **PATH Configuration**: If running `smt` in your terminal outputs `command not found`, ensure `~/.cargo/bin` (for Cargo) or `~/.local/bin` (for manually downloaded binaries) is in your shell's `PATH` environment variable:
>
> ```bash
> export PATH="$HOME/.cargo/bin:$PATH"
> ```
