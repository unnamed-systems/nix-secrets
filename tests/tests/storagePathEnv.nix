{
  pkgs,
  nixosModule,
  shared,
  ...
}:
pkgs.testers.runNixOSTest (
  { lib, ... }:
  {
    name = "storagePathEnv";

    nodes.machine = {
      imports = [
        nixosModule
        shared.minimalNoActivate
      ];

      security.nix-secrets.storagePath = "/foo/bar";
    };

    testScript =
      { nodes, ... }:
      ''
        env_var = machine.succeed("printenv NIX_SECRETS_STORAGE_PATH").strip()

        assert env_var == ${lib.strings.escapeNixString nodes.machine.security.nix-secrets.storagePath}
      '';
  }
)
