{ lib, config, ... }:
let
  cfg = config.security.nix-secrets;
in
{
  config.system.activationScripts =
    lib.mkIf (cfg.enable && cfg.activate.method == "activationScripts")
      {
        nixSecretsActivate = {
          deps = [
            "specialfs"
            "users"
            "groups"
          ];
          text = cfg.activate.command true;
          supportsDryActivation = true;
        };

        nixSecretsActivateBeforeUsers = {
          deps = [ "specialfs" ];
          text = cfg.activate.command false;
          supportsDryActivation = true;
        };

        users.deps = [ "nixSecretsActivateBeforeUsers" ];
        groups.deps = [ "nixSecretsActivateBeforeUsers" ];
      };
}
