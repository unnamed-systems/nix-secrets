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
  };

  config = lib.mkIf cfg.enable {
    environment.variables.NIX_SECRETS_NIX_EVAL_COMMAND = cfg.nixEvalCommand;
  };
}
