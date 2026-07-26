{ lib, config, ... }:
let
  cfg = config.security.nix-secrets;
in
{
  options.security.nix-secrets = {
    nixEvalCommand = lib.mkOption {
      description = "TODO";
      type = lib.types.nullOr lib.types.str;
      default = "${lib.getExe config.nix.package} --extra-experimental-features 'nix-command flakes' eval --raw";
      defaultText = lib.literalExpression "\${lib.getExe config.nix.package} --extra-experimental-features 'nix-command flakes' eval --raw";
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
        NIX_SECRETS_STORAGE_PATH = cfg.storagePath;
      };

      systemPackages = lib.optional cfg.installPackage cfg.package;
    };
  };
}
