{
  inputs,
  pkgs,
  ...
}: let
  pkgsWithOverlay = pkgs.extend inputs.self.overlays.default;
  treefmtEval = inputs.treefmt.lib.evalModule pkgsWithOverlay {
    imports = [
      ./treefmt-module.nix
      ./treefmt.nix
      inputs.pedantix.treefmtModules.default
    ];
  };
in
  treefmtEval.config.build.wrapper.overrideAttrs (old: {
    passthru =
      (old.passthru or {})
      // {
        check = treefmtEval.config.build.check inputs.self;
      };
  })
