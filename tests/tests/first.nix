{
  pkgs,
  nixosModule,
  package,
  ...
}:
pkgs.testers.runNixOSTest (
  { lib, ... }:
  {
    name = "first";

    nodes.machine = {
      environment.systemPackages = [ package ];
    };

    testScript = ''
      machine.succeed("nix-secrets")
    '';
  }
)
