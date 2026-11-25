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
rustPlatform.buildRustPackage (self: {
  # `cargoFileWorkspace` is used for `pname`, since that is the default package.
  pname = cargoFileWorkspace.workspace.package.name;
  version = cargoFileWorkspace.workspace.package.version;

  src = lib.fileset.toSource {
    root = ../.;
    fileset = lib.fileset.unions [
      ../manifest.nix
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

  nativeBuildInputs = [ installShellFiles ];

  postInstall = ''
    exe="$out/bin/${self.meta.mainProgram}"

    installShellCompletion --cmd ${self.meta.mainProgram} \
      --bash <(COMPLETE=bash $exe) \
      --fish <(COMPLETE=fish $exe) \
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
    platforms = lib.platforms.all;
    mainProgram = (builtins.head cargoFile.bin).name;
  };
})
