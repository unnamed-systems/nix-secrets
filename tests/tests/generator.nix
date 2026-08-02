{
  pkgs,
  nixosModule,
  shared,
  ...
}:
pkgs.testers.runNixOSTest (
  { lib, ... }:
  {
    name = "generator";

    nodes.machine = {
      imports = [
        shared.outPath
        nixosModule
      ];
    };

    testScript = ''
      machine.succeed("nix-secrets -f ${shared}#shared regenerate --storage $(mktemp -d) --all")
    '';
  }
)
