{
  pkgs ? import <nixpkgs> { },
}:
let
  inherit (pkgs) lib;
  fenix = pkgs.callPackage (pkgs.fetchFromGitHub {
    owner = "nix-community";
    repo = "fenix";
    rev = "42d2c5f54f90dd11ebd5dc7732f004aa900cfecd";
    hash = "sha256-uexExdN1itvHYHFek4bpvRqURvxYLKWUetpekP+udbk=";
  }) { };

  toolchain = fenix.combine [
    fenix.default.toolchain
    fenix.complete.rust-src
  ];
in
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
    nativeBuildInputs = with pkgs; [
      pkg-config
      toolchain
      llvmPackages.lld

      clippy
      taplo
      lldb

      nixfmt
    ];

    env =
      let
        inherit (pkgs.llvmPackages) llvm;
      in
      {
        RUST_SRC_PATH = "${toolchain}/lib/rustlib/src/rust/library";
        LIBCLANG_PATH = "${pkgs.llvmPackages.libclang.lib}/lib";

        LLVM_COV = "${llvm}/bin/llvm-cov";
        LLVM_PROFDATA = "${llvm}/bin/llvm-profdata";
      }
      // lib.optionalAttrs pkgs.stdenv.hostPlatform.isLinux {
        BINDGEN_EXTRA_CLANG_ARGS = "--sysroot=${pkgs.glibc.dev}";
      };
  }
) { }
