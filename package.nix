{
  lib,
  rustPlatform,
  debugBuild ? false,
}:
let
  cargoFile = (builtins.fromTOML (builtins.readFile ./Cargo.toml));
in
rustPlatform.buildRustPackage (finalAttrs: {
  pname = cargoFile.package.name;
  version = cargoFile.package.version;

  src = lib.fileset.toSource {
    root = ./.;
    fileset = lib.fileset.unions [
      ./src
      ./Cargo.toml
      ./Cargo.lock
    ];
  };

  cargoLock.lockFile = ./Cargo.lock;

  buildType = if debugBuild then "debug" else "release";

  nativeBuildInputs = [
    rustPlatform.bindgenHook
  ];

  meta = {
    description = "Postmodern secrets manager for NixOS";
    homepage = "https://github.com/unnamed-systems/nix-secrets";
    license = lib.licenses.eupl12;
    maintainers = [ lib.maintainers.bananad3v lib.maintainers.yunfachi ];
    mainProgram = cargoFile.package.name;
  };
})
