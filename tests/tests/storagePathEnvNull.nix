{
  pkgs,
  nixosModule,
  shared,
  ...
}:
pkgs.testers.runNixOSTest (
  { lib, ... }:
  {
    name = "storagePathEnvNull";

    nodes.machine = {
      imports = [
        nixosModule
        shared.minimal
      ];
    };

    testScript = ''
      machine.fail("printenv $NIX_SECRETS_STORAGE_PATH")
    '';
  }
)
