{
  pkgs,
  nixosModule,
  shared,
  ...
}:
pkgs.testers.runNixOSTest (
  { lib, ... }:
  {
    name = "nixEvalCommandEnv";

    nodes.machine = {
      imports = [ nixosModule shared.minimal ];
    };

    testScript = { nodes, ... }: ''
      env_var = machine.succeed("printenv NIX_SECRETS_NIX_EVAL_COMMAND").strip()

      assert env_var == ${lib.strings.escapeNixString nodes.machine.security.nix-secrets.nixEvalCommand}
    '';
  }
)
