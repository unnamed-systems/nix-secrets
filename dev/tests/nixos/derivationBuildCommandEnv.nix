{
  pkgs,
  nixosModule,
  shared,
  ...
}:
pkgs.testers.runNixOSTest (
  { lib, ... }:
  {
    name = "derivationBuildCommandEnv";

    nodes.machine = {
      imports = [
        nixosModule
        shared.minimalNoActivate
      ];

      security.nix-secrets.derivationBuildCommand = "foo --bar 'build'";
    };

    testScript =
      { nodes, ... }:
      ''
        env_var = machine.succeed("printenv NIX_SECRETS_DERIVATION_BUILD_COMMAND").strip()

        assert env_var == ${lib.strings.escapeNixString nodes.machine.security.nix-secrets.derivationBuildCommand}
      '';
  }
)
