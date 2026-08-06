{
  pkgs,
  nixosModule,
  shared,
  ...
}:
pkgs.testers.runNixOSTest (
  { lib, ... }:
  {
    name = "storagePathEnvUnset";

    nodes.machine = {
      imports = [
        nixosModule
        shared.minimalNoActivate
      ];

      security.nix-secrets.storagePath = null;
    };

    testScript = ''
      machine.fail("printenv $NIX_SECRETS_STORAGE_PATH")
    '';
  }
)
