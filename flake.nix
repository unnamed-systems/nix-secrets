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

      devInputs =
        (import (builtins.fetchurl {
          url = "https://raw.githubusercontent.com/NixOS/flake-compat/f275e157c50c3a9a682b4c9b4aa4db7a4cd3b5f2/default.nix";
          sha256 = "sha256:0nycwx0777d451k63ghp6p5lcv791kziqgkvqzmr1qwzywkdk1cj";
        }) { src = ./dev; }).defaultNix.inputs;

      treefmtEval = eachSystem (
        _system: pkgs:
        devInputs.treefmt-nix.lib.evalModule pkgs (_: {
          projectRootFile = "flake.nix";

          programs = {
            nixfmt.enable = true;
            statix.enable = true;
            deadnix.enable = true;
          };

          settings.formatter = {
            deadnix = {
              priority = 1;
            };

            statix = {
              priority = 2;
            };

            nixfmt = {
              priority = 3;
            };
          };
        })
      );
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
          docs = pkgs.callPackage ./docs { inherit self; };
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

      checks = eachSystem (
        system: _pkgs:
        self.packages.${system}
        // {
          formatting = treefmtEval.${system}.config.build.check self;
        }
      );

      devShells = eachSystem (
        _system: pkgs: {
          default = import ./shell.nix { inherit pkgs; };
        }
      );

      formatter = eachSystem (system: _pkgs: treefmtEval.${system}.config.build.wrapper);
    };
}
