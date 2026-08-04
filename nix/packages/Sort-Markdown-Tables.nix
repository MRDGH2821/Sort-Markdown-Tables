{
  crane ? inputs.crane or null,
  craneLib ?
    if crane != null && crane ? mkLib
    then crane.mkLib pkgs
    else null,
  inputs ? {},
  pkgs,
  ...
}: let
  builder =
    if craneLib != null
    then let
      markdownFilter = path: type: (craneLib.filterCargoSources path type) || (pkgs.lib.hasSuffix ".md" path);
      src = pkgs.lib.cleanSourceWith {
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
    else
      pkgs.rustPlatform.buildRustPackage {
        cargoLock.lockFile = ../../Cargo.lock;
        meta.mainProgram = "smt";
        pname = "Sort-Markdown-Tables";
        src = pkgs.lib.cleanSource ../..;
        version = (fromTOML (builtins.readFile ../../Cargo.toml)).package.version;
      };
in
  builder
