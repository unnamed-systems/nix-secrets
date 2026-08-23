{ lib, config, ... }:
let
  cfg = config.security.nix-secrets;
in
{
  config.launchd.daemons.nix-secrets-activate =
    lib.mkIf (cfg.enable && cfg.activate.enable && cfg.activate.method == "launchd")
      {
        command = cfg.activate.command false;
        serviceConfig = {
          RunAtLoad = true;
          KeepAlive.SuccessfulExit = false;
        };
      };

 config.launchd.daemons.nix-secrets-activate-before-users =
    lib.mkIf (cfg.enable && cfg.activate.enable && cfg.activate.method == "launchd")
      {
        command = cfg.activate.command true;
        serviceConfig = {
          RunAtLoad = true;
          KeepAlive.SuccessfulExit = false;
        };
      };
}
