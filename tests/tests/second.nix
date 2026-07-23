{
  pkgs,
  nixosModule,
  package,
  ...
}:
pkgs.testers.runNixOSTest (
  { lib, ... }:
  {
    name = "second";

    nodes.machine = {
      imports = [ nixosModule ];

      # security.nix-secrets = {
      #   enable = true;
      #   directory = ../data;
      # };
    };

    testScript = ''
      # ...
    '';
  }
)
