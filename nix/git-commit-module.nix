{
  config,
  inputs ? {},
  lib,
  pkgs,
  ...
}: let
  cfg = config.hooks.sort-markdown-tables;
  craneLib =
    if inputs ? crane
    then inputs.crane.mkLib pkgs
    else pkgs.craneLib or null;
in {
  config = lib.mkIf cfg.enable {
    hooks.sort-markdown-tables = {
      inherit (cfg) enable;
      description = "Sort markdown tables opted-in via <!-- smt --> HTML comments";
      entry = "${lib.getExe cfg.package} -i";
      files = "\\.md$";
      language = "system";
      name = "sort-markdown-tables";
      types = ["markdown"];
    };
  };
  options.hooks.sort-markdown-tables = {
    enable = lib.mkEnableOption "sort-markdown-tables pre-commit hook";
    package = lib.mkOption {
      default =
        pkgs.sort-markdown-tables or pkgs.smt or (pkgs.callPackage ./packages/Sort-Markdown-Tables.nix (
          {
            inherit inputs;
          }
          // (lib.optionalAttrs (craneLib != null) {inherit craneLib;})
        ));
      defaultText = lib.literalExpression "pkgs.sort-markdown-tables";
      description = "The sort-markdown-tables package to use.";
      type = lib.types.package;
    };
  };
}
