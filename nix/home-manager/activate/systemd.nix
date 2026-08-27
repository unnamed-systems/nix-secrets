{ lib, config, ... }:
let
  cfg = config.security.nix-secrets;
in
{
  config.systemd.user.services.nix-secrets-activate =
    lib.mkIf (cfg.enable && cfg.activate.enable && cfg.activate.method == "systemd")
      {
        Unit = {
          Description = "TODO";
        };
        Service = {
          Type = "oneshot";
          ExecStart = cfg.activate.command false;
          RemainAfterExit = true;
        };
        Install.WantedBy = [ "default.target" ];
      };
}
