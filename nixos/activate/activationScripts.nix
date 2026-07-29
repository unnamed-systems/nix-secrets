{ lib, config, ... }:
let
  cfg = config.security.nix-secrets;
  getActivationCommand = finalCommand: ''
    (
    ${lib.concatStringsSep "\n" (
      lib.mapAttrsToList (n: v: "  export ${n}='${v}'") {
        PATH = lib.makeBinPath cfg.extraPackages;
      }
    )}
      ${finalCommand}
    )
  '';
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
