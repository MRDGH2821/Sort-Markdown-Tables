{
  flake,
  pkgs,
  ...
}:
(flake.mkPackagesFor pkgs.pkgsCross.armv7l-hf-multiplatform).Sort-Markdown-Tables
