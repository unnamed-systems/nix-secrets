{ lib, config, ... }:
let
  cfg = config.security.nix-secrets;
in
{
  options.security.nix-secrets = {
    nixEvalCommand = lib.mkOption {
      description = "TODO";
      type = lib.types.nullOr lib.types.str;
      default = "${lib.getExe config.nix.package} --extra-experimental-features 'nix-command flakes' eval --raw {{input}}";
      defaultText = lib.literalExpression "\${lib.getExe config.nix.package} --extra-experimental-features 'nix-command flakes' eval --raw {{input}}";
      # example = TODO;
    };

    generatorBuildCommand = lib.mkOption {
      description = "TODO";
      type = lib.types.nullOr lib.types.str;
      default = "${config.nix.package}/bin/nix-store --realise {{input}}";
      defaultText = lib.literalExpression "\${config.nix.package}/bin/nix-store --realise {{input}}";
      # example = TODO;
    };

    storagePath = lib.mkOption {
      description = "TODO";
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
      variables = {
        NIX_SECRETS_NIX_EVAL_COMMAND = cfg.nixEvalCommand;
        NIX_SECRETS_GENERATOR_BUILD_COMMAND = cfg.generatorBuildCommand;
        NIX_SECRETS_STORAGE_PATH = cfg.storagePath;
      };

      systemPackages = lib.optional cfg.installPackage cfg.package;
    };
  };
}
