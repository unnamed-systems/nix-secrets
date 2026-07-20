{ lib, config, ... }:
let
  cfg = config.security.nix-secrets;
in
{
  # TODO only for userborn & sysusers

  config = lib.mkIf cfg.enable {
    systemd.services.nix-secrets-activate = {
      wantedBy = [ "sysinit.target" ];
      after = [ "systemd-sysusers.service" ];
      unitConfig.DefaultDependencies = "no";

      serviceConfig = {
        Type = "oneshot";
        ExecStart = "${lib.getExe cfg.package} activate --manifest ${builtins.toFile "nix-secrets-manifest.json" cfg.manifest}";
        RemainAfterExit = true;
      };
    };
  };
}
