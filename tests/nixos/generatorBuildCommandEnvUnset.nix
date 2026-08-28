{
  pkgs,
  nixosModule,
  shared,
  ...
}:
pkgs.testers.runNixOSTest {
  name = "generatorBuildCommandEnvUnset";

  nodes.machine = {
    imports = [
      nixosModule
      shared.minimalNoActivate
    ];

    security.nix-secrets.generatorBuildCommand = null;
  };

  testScript = ''
    machine.fail("printenv $NIX_SECRETS_GENERATOR_BUILD_COMMAND")
  '';
}
