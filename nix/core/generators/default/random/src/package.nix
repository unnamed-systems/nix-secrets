{ rustPlatform, ... }:
let
  cargo = builtins.fromTOML (builtins.readFile ./Cargo.toml);
in
rustPlatform.buildRustPackage {
  pname = cargo.package.name;
  inherit (cargo.package) version;

  src = ./.;
  cargoLock.lockFile = "${./.}/Cargo.lock";

  meta.mainProgram = (builtins.head cargo.bin).name;
}
