{
  config,
  lib,
  pkgs,
  ...
}: let
  cfg = config.programs.sort-markdown-tables;
in {
  options.programs.sort-markdown-tables = {
    enable = lib.mkEnableOption "sort-markdown-tables";

    package = lib.mkOption {
      type = lib.types.package;
      default = pkgs.sort-markdown-tables or pkgs.smt or (pkgs.callPackage ./packages/Sort-Markdown-Tables.nix {});
      defaultText = lib.literalExpression "pkgs.sort-markdown-tables";
      description = "The sort-markdown-tables package to use.";
    };

    includes = lib.mkOption {
      type = lib.types.listOf lib.types.str;
      default = ["*.md"];
      description = "List of files/globs to include for formatting.";
    };

    excludes = lib.mkOption {
      type = lib.types.listOf lib.types.str;
      default = [];
      description = "List of files/globs to exclude from formatting.";
    };

    priority = lib.mkOption {
      type = lib.types.nullOr lib.types.int;
      default = 2;
      description = "Formatter priority.";
    };
  };

  config = lib.mkIf cfg.enable {
    settings.formatter.sort-markdown-tables = {
      command = lib.getExe cfg.package;
      includes = cfg.includes;
      excludes = cfg.excludes;
      options = ["-i"];
    } // (lib.optionalAttrs (cfg.priority != null) { inherit (cfg) priority; });
  };
}
