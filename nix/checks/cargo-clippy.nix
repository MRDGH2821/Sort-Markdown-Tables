{
  inputs,
  lib ? pkgs.lib,
  pkgs,
  ...
}: let
  craneLib = inputs.crane.mkLib pkgs;
  markdownFilter = path: type: (craneLib.filterCargoSources path type) || (lib.hasSuffix ".md" path);
  src = lib.cleanSourceWith {
    filter = markdownFilter;
    src = craneLib.path ../..;
  };
  commonArgs = {
    inherit src;
    pname = "Sort-Markdown-Tables";
    strictDeps = true;
    version = (fromTOML (builtins.readFile ../../Cargo.toml)).package.version;
  };
  cargoArtifacts = craneLib.buildDepsOnly commonArgs;
in
  craneLib.cargoClippy (
    commonArgs
    // {
      inherit cargoArtifacts;
      cargoClippyExtraArgs = "--all-targets -- -D warnings";
    }
  )
