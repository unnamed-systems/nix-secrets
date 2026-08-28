{
  system,
  pkgs,
  lib,
  self,
  devInputs,
}:
let
  nixosModule = self.nixosModules.default;
  homeManagerModule = self.homeManagerModules.default;
  hjemModule = self.hjemModules.default;

  package = self.packages.${system}.default;

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
lib.genAttrs' files (
  file:
  lib.fix (finalAttrs: {
    name = "test-${lib.removePrefix "vm-test-run-" finalAttrs.value.name}";
    value = pkgs.callPackage file {
      inherit
        nixosModule
        homeManagerModule
        hjemModule
        package
        shared
        devInputs
        ;
    };
  })
)
