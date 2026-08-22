{
  flake,
  pkgs,
  ...
}:
(flake.mkPackagesFor pkgs.pkgsCross.mingwW64).Sort-Markdown-Tables
