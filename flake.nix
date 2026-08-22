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
        sort-markdown-tables = import ./nix/git-commit-module.nix {inherit inputs;};
      };
      treefmtModules = rec {
        default = sort-markdown-tables;
        sort-markdown-tables = import ./nix/treefmt-module.nix {inherit inputs;};
      };
    };
}
