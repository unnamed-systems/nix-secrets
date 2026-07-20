{
  description = "TODO";

  inputs = {
    nixpkgs.url = "https://nixos.org/channels/nixos-unstable/nixexprs.tar.xz";
  };

  outputs =
    { nixpkgs, self, ... }:
    let
      systems = nixpkgs.lib.systems.flakeExposed;

      eachSystem = f: nixpkgs.lib.genAttrs systems (system: f system nixpkgs.legacyPackages.${system});
    in
    {
      nixosModules = {
        default = self.nixosModules.nix-secrets;
        nix-secrets = ./nixos/default.nix;
      };

      packages = eachSystem (
        system: pkgs: {
          default = self.packages.${system}.nix-secrets;
          nix-secrets = pkgs.callPackage ./package.nix { };
        }
      );

      devShells = eachSystem (
        _system: pkgs: {
          default = import ./shell.nix { inherit pkgs; };
        }
      );
    };
}
