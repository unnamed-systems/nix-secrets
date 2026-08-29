{ inputs, lib, ... }:
let
  shared = {
    outPath = ./shared;
    default = ./shared/default.nix;
    defaultNoActivate = ./shared/defaultNoActivate.nix;
    minimal = ./shared/minimal.nix;
    minimalNoActivate = ./shared/minimalNoActivate.nix;
  };

  files = builtins.filter (lib.hasSuffix ".nix") (
    lib.fileset.toList (
      lib.fileset.unions [
        ./hjem
        ./home-manager
        ./nixos
      ]
    )
  );
in
{
  perSystem = { pkgs, system, ... }: {
    packages = lib.genAttrs' files (
      file:
      lib.fix (finalAttrs: {
        name = "test-${lib.removePrefix "vm-test-run-" finalAttrs.value.name}";
        value = pkgs.callPackage file {
          inherit
            shared
            inputs
            ;

          nixosModule = inputs.nix-secrets.nixosModules.default;
          homeManagerModule = inputs.nix-secrets.homeManagerModules.default;
          hjemModule = inputs.nix-secrets.hjemModules.default;
          package = inputs.nix-secrets.packages.${system}.default;
        };
      })
    );
  };
}
