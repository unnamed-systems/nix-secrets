{ lib, config, ... }:
{
  options.security.nix-secrets = {
    nixEvalCommand = lib.mkOption {
      description = ''
        Command used by the nix-secrets CLI to evaluate the
        `security.nix-secrets.manifest` option.

        The command must contain the `{{input}}` placeholder, which is replaced with
        the flake reference provided to the CLI followed by the manifest option path.
      '';
      type = lib.types.nullOr lib.types.str;
      default = "${config.nix.package}/bin/nix-instantiate --extra-experimental-features 'flakes' --eval --raw --expr '{{input}}'";
      defaultText = lib.literalExpression ''"''${config.nix.package}/bin/nix-instantiate --extra-experimental-features 'flakes' --eval --raw --expr '{{input}}'"'';
      # example = TODO;
    };

    generatorBuildCommand = lib.mkOption {
      description = ''
        Command used by the nix-secrets CLI to build secret generators.

        The command must contain the `{{input}}` placeholder, which is replaced with
        the derivation path of the generator.
      '';
      type = lib.types.nullOr lib.types.str;
      default = "${config.nix.package}/bin/nix-store --realise {{input}}";
      defaultText = lib.literalExpression ''"''${config.nix.package}/bin/nix-store --realise {{input}}"'';
      # example = TODO;
    };

    storagePath = lib.mkOption {
      description = ''
        Default storage path used by the nix-secrets CLI.

        The path must be absolute and outside of the Nix store.

        When set, the CLI does not require the `--storage` argument.
      '';
      type = lib.types.nullOr (
        lib.types.pathWith {
          inStore = false;
          absolute = true;
        }
      );
      default = null;
      example = "/etc/nixos/secrets";
    };

    installPackage = lib.mkOption {
      description = "Whether to enable installation of `security.nix-secrets.package` into `environment.systemPackages`.";
      type = lib.types.bool;
      default = true;
      example = false;
    };
  };
}
