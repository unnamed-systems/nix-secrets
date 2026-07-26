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
      env_var = machine.succeed("echo -n $NIX_SECRETS_NIX_EVAL_COMMAND")

      assert ${lib.strings.escapeNixString nodes.machine.security.nix-secrets.nixEvalCommand} == env_var
    '';
  }
)
