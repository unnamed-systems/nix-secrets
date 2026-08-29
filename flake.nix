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

      devFlake =
        (import (builtins.fetchurl {
          url = "https://raw.githubusercontent.com/NixOS/flake-compat/f275e157c50c3a9a682b4c9b4aa4db7a4cd3b5f2/default.nix";
          sha256 = "sha256:0nycwx0777d451k63ghp6p5lcv791kziqgkvqzmr1qwzywkdk1cj";
        }) { src = ./dev; }).defaultNix;

      devOutputs = devFlake.inputs.flake-parts.lib.mkFlake {
        inputs = devFlake.inputs // {
          nix-secrets = self;
          self = devFlake;
        };
      } ./dev/config.nix;
    in
    {
      inherit (devOutputs) debug checks formatter;

      nixosModules = {
        default = self.nixosModules.nix-secrets;
        nix-secrets = ./nix/nixos/default.nix;
      };

      darwinModules = {
        default = self.darwinModules.nix-secrets;
        nix-secrets = ./nix/nix-darwin/default.nix;
      };

      homeManagerModules = {
        default = self.homeManagerModules.nix-secrets;
        nix-secrets = ./nix/home-manager/default.nix;
      };

      hjemModules = {
        default = self.hjemModules.nix-secrets;
        nix-secrets = ./nix/hjem/default.nix;
      };

      packages = eachSystem (
        system: pkgs: {
          default = self.packages.${system}.nix-secrets;
          nix-secrets = pkgs.callPackage ./package.nix { };
          docs = pkgs.callPackage ./docs { inherit self; };

          # nix eval --impure --raw .#debug.flake.packages.x86_64-linux --apply 'x: builtins.concatStringsSep " " (builtins.attrNames x)'
          inherit (devOutputs.packages.${system})
            test-activate
            test-decrypt
            test-decryptRejectsUnmanagedSecrets
            test-edit
            test-generator
            test-generatorBuildCommandEnv
            test-generatorBuildCommandEnvStandalone
            test-generatorBuildCommandEnvUnset
            test-hjem-activate
            test-home-manager-activate
            test-installPackageFalse
            test-installPackageTrue
            test-keygen
            test-nixEvalCommandEnv
            test-nixEvalCommandEnvStandalone
            test-nixEvalCommandEnvUnset
            test-rekey
            test-storagePathEnvStandalone
            test-templateNoRecursiveSecretRendering
            ;
        }
      );
    };

  nixConfig = {
    extra-substituters = [ "https://nix-secrets.cachix.org" ];
    extra-trusted-public-keys = [
      "nix-secrets.cachix.org-1:NSwybk1LexO4kPH755itLM1t2NGegVq9YR22KlG8Vp0="
    ];
  };
}
