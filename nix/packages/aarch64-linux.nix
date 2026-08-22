{
  flake,
  pkgs,
  ...
}:
(flake.mkPackagesFor pkgs.pkgsCross.aarch64-multiplatform).Sort-Markdown-Tables
