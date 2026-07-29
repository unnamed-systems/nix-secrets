{ lib, config, ... }:
let
  cfg = config.security.nix-secrets;
in
{
  config.systemd.services = lib.mkIf (cfg.enable && cfg.activate.method == "systemd") {
    nix-secrets-activate = {
      wantedBy = [ "sysinit.target" ];
      after = [
        "local-fs.target"
        "systemd-sysusers.service"
        "userborn.service"
      ];
      requiredBy = [ "sysinit-reactivation.target" ];
      before = [ "sysinit-reactivation.target" ];
      unitConfig.DefaultDependencies = "no";
      environment.PATH = lib.makeBinPath cfg.extraPackages;

      serviceConfig = {
        Type = "oneshot";
        ExecStart = cfg.activate.command false;
        RemainAfterExit = true;
      };
      # unitConfig.RequiresMountsFor
    };

    nix-secrets-activate-before-users = {
      wantedBy = [ "systemd-sysusers.service" ];
      before = [ "systemd-sysusers.service" ];
      unitConfig.DefaultDependencies = "no";
      environment.PATH = lib.makeBinPath cfg.extraPackages;

      serviceConfig = {
        Type = "oneshot";
        ExecStart = cfg.activate.command true;
        RemainAfterExit = true;
      };
      # unitConfig.RequiresMountsFor
    };
  };
}
