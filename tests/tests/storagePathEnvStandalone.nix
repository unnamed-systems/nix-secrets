{
  pkgs,
  nixosModule,
  shared,
  ...
}:
pkgs.testers.runNixOSTest (
  { lib, ... }:
  {
    name = "storagePathEnvStandalone";

    nodes.machine = {
      imports = [
        nixosModule
        shared.minimalNoActivate
      ];

      security.nix-secrets.storagePath = null;

      environment.variables.NIX_SECRETS_STORAGE_PATH = "/foo/bar";
    };

    testScript =
      { nodes, ... }: ''
        env_var = machine.succeed("printenv NIX_SECRETS_STORAGE_PATH").strip()

        assert env_var == ${lib.strings.escapeNixString nodes.machine.environment.variables.NIX_SECRETS_STORAGE_PATH}
      '';
  }
)
