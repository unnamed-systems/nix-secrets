{ lib, config, ... }:
let
  cfg = config.security.nix-secrets;
in
{
  config = lib.mkIf cfg.enable {
    environment.variables.NIX_SECRETS_NIX_EVAL_COMMAND = cfg.nixEvalCommand;
  };
}
