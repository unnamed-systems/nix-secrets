{
  lib,
  rustPlatform,
}:
let
  cargoFile = (builtins.fromTOML (builtins.readFile ./Cargo.toml));
in
rustPlatform.buildRustPackage (finalAttrs: {
  pname = cargoFile.package.name;
  version = cargoFile.package.version;

  src = ./.;

  cargoLock.lockFile = ./Cargo.lock;

  meta = {
    description = "TODO";
    homepage = "https://github.com/unnamed-systems/nix-secrets";
    license = lib.licenses.mit;
    maintainers = [ lib.maintainers.yunfachi ];
    mainProgram = cargoFile.package.name;
  };
})
