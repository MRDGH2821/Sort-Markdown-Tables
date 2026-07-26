{
  inputs,
  pkgs,
  ...
}:
import ./Sort-Markdown-Tables.nix {
  inherit inputs;
  pkgs = pkgs.pkgsCross.armv7l-hf-multiplatform;
}
