{
  pkgs,
  nixosModule,
  shared,
  ...
}:
pkgs.testers.runNixOSTest {
  name = "derivationBuildCommandEnvUnset";

  nodes.machine = {
    imports = [
      nixosModule
      shared.minimalNoActivate
    ];

    security.nix-secrets.derivationBuildCommand = null;
  };

  testScript = ''
    machine.fail("printenv $NIX_SECRETS_DERIVATION_BUILD_COMMAND")
  '';
}
