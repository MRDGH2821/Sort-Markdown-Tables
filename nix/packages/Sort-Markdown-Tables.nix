{
  craneLib ? inputs.crane.mkLib pkgs,
  inputs ? {},
  lib ? pkgs.lib,
  pkgs,
  ...
}: let
  markdownFilter = path: type: (craneLib.filterCargoSources path type) || (lib.hasSuffix ".md" path);
  src = lib.cleanSourceWith {
    filter = markdownFilter;
    src = craneLib.path ../..;
  };
  commonArgs = {
    inherit src;
    meta.mainProgram = "smt";
    pname = "Sort-Markdown-Tables";
    strictDeps = true;
    version = (fromTOML (builtins.readFile ../../Cargo.toml)).package.version;
  };
  cargoArtifacts = craneLib.buildDepsOnly commonArgs;
in
  craneLib.buildPackage (
    commonArgs
    // {
      inherit cargoArtifacts;
    }
  )
