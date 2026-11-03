{ lib, config, ... }:
let
  cfg = config.security.nix-secrets;
in
{
  config.systemd.user.services.nix-secrets-activate =
    lib.mkIf (cfg.enable && cfg.activate.enable && cfg.activate.method == "systemd")
      {
        Unit = {
          Description = "Activate nix-secrets secrets and templates";
        };
        Service = {
          Type = "oneshot";
          ExecStart = cfg.activate.command false;
          Environment = [ "PATH=${lib.makeBinPath cfg.extraPackages}" ];
          RemainAfterExit = true;
        };
        Install.WantedBy = [ "default.target" ];
      };
}
