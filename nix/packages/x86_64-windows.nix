{
  inputs,
  pkgs,
  ...
}:
import ./Sort-Markdown-Tables.nix {
  inherit inputs;
  pkgs = pkgs.pkgsCross.mingwW64;
}
