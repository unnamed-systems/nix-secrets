{
  pkgs,
  nixosModule,
  shared,
  ...
}:
pkgs.testers.runNixOSTest (
  { lib, ... }:
  {
    name = "nixEvalCommandEnvNull";

    nodes.machine = {
      imports = [ nixosModule shared.minimal ];

      security.nix-secrets.nixEvalCommand = null;
    };

    testScript = ''
      machine.fail("printenv $NIX_SECRETS_NIX_EVAL_COMMAND")
    '';
  }
)
