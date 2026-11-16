{
  lib,
  rustPlatform,
  debugBuild ? false,
}:
let
  cargoFile = builtins.fromTOML (builtins.readFile ../Cargo.toml);
in
rustPlatform.buildRustPackage {
  pname = "nix-secrets-activate";
  version = cargoFile.workspace.package.version;

  src = lib.fileset.toSource {
    root = ../.;
    fileset = lib.fileset.unions [
      ../crates
      ../Cargo.toml
      ../Cargo.lock
    ];
  };
  cargoFlags = [
    "-p"
    "nix-secrets-activate"
  ];
  cargoTestFlags = [
    "-p"
    "nix-secrets-activate"
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
    mainProgram = "nix-secrets-activate";
  };
}
