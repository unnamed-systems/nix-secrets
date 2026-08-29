{
  perSystem = { pkgs, ... }: {
    devShells.default = pkgs.callPackage (
      {
        mkShell,
        pkg-config,
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
          pkg-config
          rustPlatform.bindgenHook

          rustc
          cargo
          rustfmt
          clippy
          rust-analyzer
        ];

        RUST_SRC_PATH = "${rustPlatform.rustLibSrc}";
      }
    ) { };
  };
}
