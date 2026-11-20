{
  lib,
  rustPlatform,
  debugBuild ? false,
}:
let
  cargoFileWorkspace = builtins.fromTOML (builtins.readFile ../Cargo.toml);
  cargoFile = builtins.fromTOML (builtins.readFile ../crates/nix-secrets-activate/Cargo.toml);
in
rustPlatform.buildRustPackage {
  pname = cargoFile.package.name;
  version = cargoFileWorkspace.workspace.package.version;

  src = lib.fileset.toSource {
    root = ../.;
    fileset = lib.fileset.unions [
      ../crates
      ../Cargo.toml
      ../Cargo.lock
    ];
  };
  cargoBuildFlags = [
    "-p"
    cargoFile.package.name
  ];
  cargoTestFlags = [
    "-p"
    cargoFile.package.name
  ];

  cargoLock.lockFile = ../Cargo.lock;

  buildType = if debugBuild then "debug" else "release";

  meta = {
    description = "Postmodern secrets manager for NixOS, activation stage";
    homepage = "https://github.com/unnamed-systems/nix-secrets";
    license = lib.licenses.eupl12;
    maintainers = [
      lib.maintainers.bananad3v
      lib.maintainers.yunfachi
    ];
    platforms = lib.platforms.all;
    mainProgram = (builtins.head cargoFile.bin).name;
  };
}
