{ lib, config, ... }:
let
  cfg = config.security.nix-secrets;
  getActivationCommand = finalCommand: ''
    (
      export PATH=${lib.escapeShellArg (lib.makeBinPath cfg.extraPackages)};
      ${finalCommand} $([ "$NIXOS_ACTION" = "dry-activate" ] && echo "--dry")
    )
  '';
in
{
  config.system.activationScripts =
    lib.mkIf (cfg.enable && cfg.activate.enable && cfg.activate.method == "activationScripts")
      {
        nixSecretsActivate = {
          deps = [
            "specialfs"
            "users"
            "groups"
          ];
          text = getActivationCommand (cfg.activate.command false);
          supportsDryActivation = true;
        };

        nixSecretsActivateBeforeUsers = {
          deps = [ "specialfs" ];
          text = getActivationCommand (cfg.activate.command true);
          supportsDryActivation = true;
        };

        users.deps = [ "nixSecretsActivateBeforeUsers" ];
        groups.deps = [ "nixSecretsActivateBeforeUsers" ];
      };
}
