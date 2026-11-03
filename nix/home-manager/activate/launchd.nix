{ lib, config, ... }:
let
  cfg = config.security.nix-secrets;
in
{
  config.launchd.agents.nix-secrets-activate =
    lib.mkIf (cfg.enable && cfg.activate.enable && cfg.activate.method == "launchd")
      {
        enable = true;
        config = {
          Program = cfg.activate.command false;
          EnvironmentVariables.PATH = lib.makeBinPath cfg.extraPackages;
          RunAtLoad = true;
          KeepAlive.SuccessfulExit = false;
        };
      };
}
