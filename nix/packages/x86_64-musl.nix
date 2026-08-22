{
  flake,
  pkgs,
  ...
}:
(flake.mkPackagesFor pkgs.pkgsCross.musl64).Sort-Markdown-Tables
