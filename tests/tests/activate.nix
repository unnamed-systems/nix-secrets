{
  pkgs,
  nixosModule,
  package,
  ...
}:
pkgs.testers.runNixOSTest (
  { lib, ... }:
  {
    name = "activate";

    nodes.machine = { config, ... }: {
      imports = [
        ../shared
        nixosModule
      ];

      environment.systemPackages = [ package ];
    };

    testScript = { nodes, ... }: ''
      ${lib.concatMapStringsSep "\n"
        (
          v:
          ''machine.succeed ("test -f ${lib.removePrefix "\"" (lib.removeSuffix "\"" (lib.strings.escapeNixString (lib.strings.escapeNixString v.path)))}")''
        )
        (
          (builtins.attrValues nodes.machine.security.nix-secrets.secrets)
          ++ (builtins.attrValues nodes.machine.security.nix-secrets.templates)
        )
      }
    '';
  }
)
