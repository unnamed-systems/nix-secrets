{ lib, config, ... }:
let
  cfg = config.security.nix-secrets;
in
{
  config.systemd.services.nix-secrets-activate =
    lib.mkIf (cfg.enable && cfg.activate.enable && cfg.activate.method == "systemd")
      {
        description = "Activate nix-secrets secrets and templates";
        serviceConfig = {
          Type = "oneshot";
          ExecStart = cfg.activate.command false;
          RemainAfterExit = true;
        };
        wantedBy = [ "default.target" ];
      };
}
