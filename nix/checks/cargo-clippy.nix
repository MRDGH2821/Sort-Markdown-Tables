{
  inputs,
  pkgs,
  ...
}: let
  craneLib = inputs.crane.mkLib pkgs;
  markdownFilter = path: type: (craneLib.filterCargoSources path type) || (pkgs.lib.hasSuffix ".md" path);
  src = pkgs.lib.cleanSourceWith {
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
