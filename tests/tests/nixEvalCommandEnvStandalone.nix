{
  pkgs,
  nixosModule,
  shared,
  ...
}:
pkgs.testers.runNixOSTest (
  { lib, ... }:
  {
    name = "nixEvalCommandEnvStandalone";

    nodes.machine = {
      imports = [
        nixosModule
        shared.minimalNoActivate
      ];

      security.nix-secrets.nixEvalCommand = null;

      environment.variables.NIX_SECRETS_NIX_EVAL_COMMAND = "foo --bar 'eval'";
    };

    testScript = { nodes, ... }: ''
      env_var = machine.succeed("printenv NIX_SECRETS_NIX_EVAL_COMMAND").strip()

      assert env_var == ${lib.strings.escapeNixString nodes.machine.environment.variables.NIX_SECRETS_NIX_EVAL_COMMAND}
    '';
  }
)
