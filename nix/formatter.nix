{
  inputs,
  pkgs,
  ...
}: let
  treefmtEval = inputs.treefmt.lib.evalModule pkgs {
    _module.args = {inherit inputs;};
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
