{
  inputs,
  pkgs,
  ...
}: let
  pre-commit-check = import ./checks/pre-commit-check.nix {inherit inputs pkgs;};
  llm-pkgs = inputs.llm-agents.packages.${pkgs.stdenv.hostPlatform.system};
in
  pkgs.mkShell {
    inherit (pre-commit-check) shellHook;
    packages = [
      llm-pkgs.antigravity-cli
      llm-pkgs.apm
      llm-pkgs.copilot-cli
      llm-pkgs.cursor-agent
      llm-pkgs.git-surgeon
      llm-pkgs.opencode
      llm-pkgs.rtk
      pkgs.nil
      pkgs.nixd
    ];
  }
