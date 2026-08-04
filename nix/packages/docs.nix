{pkgs, ...}:
pkgs.stdenv.mkDerivation {
  buildPhase = ''
    runHook preBuild
    jekyll build --source docs --destination _site
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
    filter = path: _type: (pkgs.lib.hasPrefix (toString ../../docs) path);
    src = ../..;
  };
  version = (fromTOML (builtins.readFile ../../Cargo.toml)).package.version;
}
