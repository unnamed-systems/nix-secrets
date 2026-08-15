{
  description = "A postmodern secrets manager for NixOS";

  inputs = {
    nixpkgs.url = "https://nixos.org/channels/nixos-unstable/nixexprs.tar.xz";
  };

  outputs =
    {
      nixpkgs,
      self,
      ...
    }:
    let
      inherit (nixpkgs) lib;

      systems = lib.systems.flakeExposed;

      eachSystem = f: lib.genAttrs systems (system: f system nixpkgs.legacyPackages.${system});
    in
    {
      nixosModules = {
        default = self.nixosModules.nix-secrets;
        nix-secrets = ./nix/nixos/default.nix;
      };

      darwinModules = {
        default = self.darwinModules.nix-secrets;
        nix-secrets = ./nix/nix-darwin/default.nix;
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

      checks = eachSystem (system: _pkgs: self.packages.${system});

      devShells = eachSystem (
        _system: pkgs: {
          default = import ./shell.nix { inherit pkgs; };
        }
      );

      formatter = eachSystem (_system: pkgs: pkgs.nixfmt-tree);
    };
}
