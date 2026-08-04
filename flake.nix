{
  description = "Sort Markdown Tables dev shell";
  inputs = {
    blueprint = {
      inputs.nixpkgs.follows = "nixpkgs";
      url = "github:numtide/blueprint";
    };
    crane.url = "github:ipetkov/crane";
    git-hooks = {
      inputs.nixpkgs.follows = "nixpkgs";
      url = "github:cachix/git-hooks.nix";
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
    (inputs.blueprint {
      inherit inputs;
      prefix = "nix/";
    })
    // {
      gitHooksModules = rec {
        default = sort-markdown-tables;
        sort-markdown-tables = {
          _file = toString ./nix/git-commit-module.nix;
          imports = [
            (_: {
              _module.args.inputs = inputs;
            })
            ./nix/git-commit-module.nix
          ];
        };
      };
      overlays = rec {
        default = sort-markdown-tables;
        sort-markdown-tables = final: _prev: {
          craneLib = inputs.crane.mkLib final;
          smt = final.callPackage ./nix/packages/Sort-Markdown-Tables.nix {};
          sort-markdown-tables = final.smt;
        };
      };
      treefmtModules = rec {
        default = sort-markdown-tables;
        sort-markdown-tables = {
          _file = toString ./nix/treefmt-module.nix;
          imports = [
            (_: {
              _module.args.inputs = inputs;
            })
            ./nix/treefmt-module.nix
          ];
        };
      };
    };
}
