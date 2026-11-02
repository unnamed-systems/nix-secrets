{ lib, config, ... }:
{
  options.security.nix-secrets = {
    nixBuildCommand = lib.mkOption {
      description = ''
        Command used by the nix-secrets CLI to build the manifest for the
        `security.nix-secrets.manifest` option.

        The command must contain the `{{input}}` placeholder, which is replaced with
        the Nix expression used to retrieve the manifest from the flake reference
        provided to the CLI.
      '';
      type = lib.types.nullOr lib.types.str;
      default = "${
        lib.optionalString (config.nix.package or null != null) "${config.nix.package}/bin/"
      }nix-build --extra-experimental-features 'flakes' --no-out-link --expr '{{input}}'";
      defaultText = lib.literalExpression ''"''${lib.optionalString (config.nix.package or null != null) "''${config.nix.package}/bin/"}/bin/nix-build --extra-experimental-features 'flakes' --no-out-link --expr '{{input}}'"'';
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
