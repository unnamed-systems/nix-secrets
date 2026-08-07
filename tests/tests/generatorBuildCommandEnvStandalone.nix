{
  pkgs,
  nixosModule,
  shared,
  ...
}:
pkgs.testers.runNixOSTest (
  { lib, ... }:
  {
    name = "generatorBuildCommandEnvStandalone";

    nodes.machine = {
      imports = [
        nixosModule
        shared.minimalNoActivate
      ];

      security.nix-secrets.generatorBuildCommand = null;

      environment.variables.NIX_SECRETS_GENERATOR_BUILD_COMMAND = "foo --bar 'build'";
    };

    testScript =
      { nodes, ... }: ''
        env_var = machine.succeed("printenv NIX_SECRETS_GENERATOR_BUILD_COMMAND").strip()

        assert env_var == ${lib.strings.escapeNixString nodes.machine.environment.variables.NIX_SECRETS_GENERATOR_BUILD_COMMAND}
      '';
  }
)
