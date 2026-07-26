{
  inputs,
  pkgs,
  ...
}: let
  pre-commit-check = import ./checks/pre-commit-check.nix {inherit inputs pkgs;};
  llm-pkgs = inputs.llm-agents.packages.${pkgs.stdenv.hostPlatform.system};
  cross-build-all = pkgs.writeShellScriptBin "cross-build-all" ''
    set -euo pipefail
    JOBS="''${1:-4}"
    echo "===> Cross-compiling release targets from .github/workflows/release.yml (Jobs: $JOBS)..."
    ${pkgs.yq}/bin/yq '.jobs.build.strategy.matrix.include[].target' .github/workflows/release.yml \
      | grep -v "apple-darwin" \
      | xargs -I {} -P "$JOBS" cross build --release --target {}
  '';
in
  pkgs.mkShell {
    inherit (pre-commit-check) shellHook;
    packages = [
      cross-build-all
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
