{
  system,
  pkgs,
  lib,
  self,
}:
let
  nixosModule = self.nixosModules.default;
  package = self.packages.${system}.default;
  shared = {
    outPath = ./shared;
    minimal = ./shared/minimal.nix;
  };

  files = lib.fileset.toList (lib.fileset.fileFilter (file: file.hasExt "nix") ./tests);
in
lib.genAttrs' files (
  file:
  lib.fix (finalAttrs: {
    name = "test-${lib.removePrefix "vm-test-run-" finalAttrs.value.name}";
    value = pkgs.callPackage file {
      inherit nixosModule package shared;
    };
  })
)
