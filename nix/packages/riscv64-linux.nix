{
  flake,
  pkgs,
  ...
}:
(flake.mkPackagesFor pkgs.pkgsCross.riscv64).Sort-Markdown-Tables
