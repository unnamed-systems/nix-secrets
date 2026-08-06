{ lib, config, ... }:
let
  cfg = config.security.nix-secrets;
in
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
      default = "${lib.getExe config.nix.package} --extra-experimental-features 'nix-command flakes' eval --raw {{input}}";
      defaultText = lib.literalExpression "\${lib.getExe config.nix.package} --extra-experimental-features 'nix-command flakes' eval --raw {{input}}";
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
      defaultText = lib.literalExpression "\${config.nix.package}/bin/nix-store --realise {{input}}";
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

    installPackage =
      lib.mkEnableOption "installation of `security.nix-secrets.package` into `environment.systemPackages`"
      // {
        default = true;
      };
  };

  config = lib.mkIf cfg.enable {
    environment = {
      variables =
        let
          mkIfNotNull = value: lib.mkIf (value != null) value;
        in
        {
          NIX_SECRETS_NIX_EVAL_COMMAND = mkIfNotNull cfg.nixEvalCommand;
          NIX_SECRETS_GENERATOR_BUILD_COMMAND = mkIfNotNull cfg.generatorBuildCommand;
          NIX_SECRETS_STORAGE_PATH = mkIfNotNull cfg.storagePath;
        };

      systemPackages = lib.optional cfg.installPackage cfg.package;
    };
  };
}
