{pkgs, ...}:
pkgs.stdenv.mkDerivation {
  buildPhase = ''
    runHook preBuild
    jekyll build --destination _site
    runHook postBuild
  '';
  installPhase = ''
    runHook preInstall
    cp -r _site $out
    runHook postInstall
  '';
  nativeBuildInputs = [
    pkgs.jekyll
  ];
  pname = "Sort-Markdown-Tables-docs";
  src = pkgs.lib.cleanSourceWith {
    filter = path: _type:
      (pkgs.lib.hasPrefix (toString ../../docs) path) || (path == toString ../../_config.yml);
    src = ../..;
  };
  version = (fromTOML (builtins.readFile ../../Cargo.toml)).package.version;
}
