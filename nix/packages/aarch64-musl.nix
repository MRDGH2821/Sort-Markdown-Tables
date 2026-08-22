{
  flake,
  pkgs,
  ...
}:
(flake.mkPackagesFor pkgs.pkgsCross.aarch64-multiplatform-musl).Sort-Markdown-Tables
