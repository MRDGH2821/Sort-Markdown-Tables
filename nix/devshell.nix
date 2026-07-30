{
  inputs,
  pkgs,
  ...
}: let
  pre-commit-check = import ./checks/pre-commit-check.nix {inherit inputs pkgs;};
  yqExe = pkgs.lib.getExe pkgs.yq;
  crossExe = pkgs.lib.getExe pkgs.cargo-cross;
  cross-build-all = pkgs.writeShellScriptBin "cross-build-all" ''
    set -euo pipefail
    JOBS="''${1:-4}"
    echo "===> Cross-compiling release targets from .github/workflows/release.yml (Jobs: $JOBS)..."
    ${yqExe} -r '.jobs.build.strategy.matrix.include[].target' .github/workflows/release.yml \
      | grep -v "apple-darwin" \
      | xargs -I {} -P "$JOBS" ${crossExe} build --release --target {}
  '';
  cross-build-seq = pkgs.writeShellScriptBin "cross-build-seq" ''
    set -euo pipefail
    TARGETS=($(${yqExe} -r '.jobs.build.strategy.matrix.include[].target' .github/workflows/release.yml | grep -v "apple-darwin"))
    TOTAL=''${#TARGETS[@]}
    echo "===> Sequential cross-compilation for $TOTAL targets from .github/workflows/release.yml..."
    COUNT=0
    FAILED=()

    for target in "''${TARGETS[@]}"; do
      COUNT=$((COUNT + 1))
      echo "------------------------------------------------------------"
      echo "[$COUNT/$TOTAL] Building release binary for: $target"
      echo "------------------------------------------------------------"
      if ${crossExe} build --release --target "$target"; then
        echo "✓ Finished $target successfully"
      else
        echo "✗ Failed $target"
        FAILED+=("$target")
      fi
    done

    echo "============================================================"
    echo "Summary: $((TOTAL - ''${#FAILED[@]}))/$TOTAL target builds succeeded."
    if [ ''${#FAILED[@]} -gt 0 ]; then
      echo "Failed targets: ''${FAILED[*]}"
      exit 1
    fi
  '';
in
  pkgs.mkShell {
    inherit (pre-commit-check) shellHook;
    packages = [
      (import ./formatter.nix {inherit inputs pkgs;})
      cross-build-all
      cross-build-seq

      pkgs.nil
      pkgs.nixd
      pkgs.act
    ];
  }
