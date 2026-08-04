# Installation

## From source (requires Rust 1.70+)

```bash
cargo install --git https://github.com/MRDGH2821/Sort-Markdown-Tables
```

Or download a pre-built binary from [releases](https://github.com/MRDGH2821/Sort-Markdown-Tables/releases).

## Using Nix Flakes

Run directly without installing:

```bash
nix run github:MRDGH2821/Sort-Markdown-Tables -- --help
```

Or add as a flake input in `flake.nix`:

```nix
inputs.smt = {
  url = "github:MRDGH2821/Sort-Markdown-Tables";
  inputs.nixpkgs.follows = "nixpkgs";
};
```

And add `inputs.smt.packages.${system}.default` to `environment.systemPackages` or `devShells`.
