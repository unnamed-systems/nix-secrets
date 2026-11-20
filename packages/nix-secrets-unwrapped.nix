{
  lib,
  rustPlatform,
  installShellFiles,
  debugBuild ? false,
}:
let
  cargoFileWorkspace = builtins.fromTOML (builtins.readFile ../Cargo.toml);
  cargoFile = builtins.fromTOML (builtins.readFile ../crates/nix-secrets-cli/Cargo.toml);
in
rustPlatform.buildRustPackage {
  # `cargoFileWorkspace` is used for `pname`, since that is the default package.
  pname = cargoFileWorkspace.workspace.package.name;
  version = cargoFileWorkspace.workspace.package.version;

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
    cargoFile.package.name
  ];
  cargoTestFlags = [
    "-p"
    cargoFile.package.name
  ];

  cargoLock.lockFile = ../Cargo.lock;

  buildType = if debugBuild then "debug" else "release";

  nativeBuildInputs = [
    installShellFiles
    rustPlatform.bindgenHook
  ];

  postInstall = ''
    local exe="$out/bin/nix-secrets"

    installShellCompletion --cmd nix-secrets \
      --bash <(COMPLETE=bash $exe) \
      --zsh <(COMPLETE=zsh $exe)
  '';

  meta = {
    description = "Postmodern secrets manager for NixOS";
    homepage = "https://github.com/unnamed-systems/nix-secrets";
    license = lib.licenses.eupl12;
    maintainers = [
      lib.maintainers.bananad3v
      lib.maintainers.yunfachi
    ];
    mainProgram = (builtins.head cargoFile.bin).name;
  };
}
