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
      machine.fail("printenv $NIX_SECRETS_GENERATOR_BUILD_COMMAND")
    '';
  }
)
