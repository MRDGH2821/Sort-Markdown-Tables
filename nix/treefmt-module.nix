{inputs}: {
  config,
  lib,
  pkgs,
  ...
}: let
  cfg = config.programs.sort-markdown-tables;
  craneLib = inputs.crane.mkLib pkgs;
in {
  config = lib.mkIf cfg.enable {
    settings.formatter.sort-markdown-tables =
      {
        inherit (cfg) excludes;
        inherit (cfg) includes;
        command = lib.getExe cfg.package;
        options = ["-i"];
      }
      // (lib.optionalAttrs (cfg.priority != null) {inherit (cfg) priority;});
  };
  options.programs.sort-markdown-tables = {
    enable = lib.mkEnableOption "sort-markdown-tables";
    excludes = lib.mkOption {
      default = [];
      description = "List of files/globs to exclude from formatting.";
      type = lib.types.listOf lib.types.str;
    };
    includes = lib.mkOption {
      default = ["*.md"];
      description = "List of files/globs to include for formatting.";
      type = lib.types.listOf lib.types.str;
    };
    package = lib.mkOption {
      default =
        pkgs.sort-markdown-tables or pkgs.smt
          or (pkgs.callPackage ./packages/Sort-Markdown-Tables.nix {inherit craneLib;});
      defaultText = lib.literalExpression "pkgs.sort-markdown-tables";
      description = "The sort-markdown-tables package to use.";
      type = lib.types.package;
    };
    priority = lib.mkOption {
      default = 2;
      description = "Formatter priority.";
      type = lib.types.nullOr lib.types.int;
    };
  };
}
