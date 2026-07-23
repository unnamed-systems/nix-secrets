{
  description = "A postmodern secrets manager for NixOS";

  inputs = {
    nixpkgs.url = "https://nixos.org/channels/nixos-unstable/nixexprs.tar.xz";
  };

  outputs =
    { nixpkgs, self, ... }:
    let
      lib = nixpkgs.lib;

      systems = lib.systems.flakeExposed;

      eachSystem = f: lib.genAttrs systems (system: f system nixpkgs.legacyPackages.${system});
    in
    {
      nixosModules = {
        default = self.nixosModules.nix-secrets;
        nix-secrets = ./nixos/default.nix;
      };

      packages = eachSystem (
        system: pkgs:
        {
          default = self.packages.${system}.nix-secrets;
          nix-secrets = pkgs.callPackage ./package.nix { };
        }
        // import ./tests {
          inherit
            system
            pkgs
            lib
            self
            ;
        }
      );

      devShells = eachSystem (
        _system: pkgs: {
          default = import ./shell.nix { inherit pkgs; };
        }
      );
    };
}
