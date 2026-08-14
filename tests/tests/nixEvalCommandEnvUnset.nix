{
  pkgs,
  nixosModule,
  shared,
  lib,
  ...
}:
pkgs.testers.runNixOSTest {
  name = "nixEvalCommandEnvUnset";

  nodes.machine = {
    imports = [
      nixosModule
      shared.minimalNoActivate
    ];

    security.nix-secrets.nixEvalCommand = lib.mkForce null;
  };

  testScript = ''
    machine.fail("printenv $NIX_SECRETS_NIX_EVAL_COMMAND")
  '';
}
