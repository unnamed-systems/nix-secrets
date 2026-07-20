{
  pkgs ? import <nixpkgs> { },
}:
pkgs.callPackage (
  {
    mkShell,
    rustc,
    cargo,
    rustPlatform,
    rustfmt,
    clippy,
    rust-analyzer,
  }:
  mkShell {
    strictDeps = true;
    nativeBuildInputs = [
      rustc
      cargo
      rustfmt
      clippy
      rust-analyzer
    ];

    RUST_SRC_PATH = "${rustPlatform.rustLibSrc}";
  }
) { }
