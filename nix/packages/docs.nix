{pkgs, ...}:
pkgs.stdenv.mkDerivation {
  buildPhase = ''
    runHook preBuild
    zensical build --clean
    runHook postBuild
  '';
  installPhase = ''
    runHook preInstall
    cp -r site $out
    runHook postInstall
  '';
  nativeBuildInputs = [
    pkgs.zensical
  ];
  pname = "Sort-Markdown-Tables-docs";
  src = pkgs.lib.cleanSourceWith {
    filter = path: _type:
      (pkgs.lib.hasPrefix (toString ../../docs) path) || (path == toString ../../zensical.toml);
    src = ../..;
  };
  version = (fromTOML (builtins.readFile ../../Cargo.toml)).package.version;
}
