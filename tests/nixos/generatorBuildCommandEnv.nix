{
  pkgs,
  nixosModule,
  shared,
  ...
}:
pkgs.testers.runNixOSTest (
  { lib, ... }:
  {
    name = "generatorBuildCommandEnv";

    nodes.machine = {
      imports = [
        nixosModule
        shared.minimalNoActivate
      ];

      security.nix-secrets.generatorBuildCommand = "foo --bar 'build'";
    };

    testScript =
      { nodes, ... }:
      ''
        env_var = machine.succeed("printenv NIX_SECRETS_GENERATOR_BUILD_COMMAND").strip()

        assert env_var == ${lib.strings.escapeNixString nodes.machine.security.nix-secrets.generatorBuildCommand}
      '';
  }
)
