{
  pkgs,
  nixosModule,
  shared,
  ...
}:
pkgs.testers.runNixOSTest (
  { lib, ... }:
  {
    name = "generatorBuildCommandEnvNull";

    nodes.machine = {
      imports = [
        nixosModule
        shared.minimal
      ];

      security.nix-secrets.generatorBuildCommand = null;
    };

    testScript = { nodes, ... }: ''
      env_var = machine.succeed("printenv NIX_SECRETS_GENERATOR_BUILD_COMMAND").strip()

      assert env_var == ${lib.strings.escapeNixString nodes.machine.security.nix-secrets.generatorBuildCommand}
    '';
  }
)
