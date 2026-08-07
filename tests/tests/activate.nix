{
  pkgs,
  nixosModule,
  shared,
  ...
}:
pkgs.testers.runNixOSTest (
  { lib, ... }:
  {
    name = "activate";

    nodes.machine = { config, ... }: {
      imports = [
        shared.outPath
        nixosModule
      ];
    };

    testScript =
      { nodes, ... }:
      let
        checks =
          map
            (secret: {
              path = secret.path;
              mode = lib.removePrefix "0" secret.mode;
              owner = toString secret.owner;
              group = toString secret.group;
            })
            (
              (builtins.attrValues nodes.machine.security.nix-secrets.secrets)
              ++ (builtins.attrValues nodes.machine.security.nix-secrets.templates)
            );
      in
      ''
        import json
        import shlex

        checks = json.loads(${builtins.toJSON (builtins.toJSON checks)})

        def stat(path, fmt):
          return machine.succeed(
            f"stat -c {shlex.quote(fmt)} {shlex.quote(path)}"
          ).strip()

        def check_stat(path, fmt, expected):
          actual = stat(path, fmt)
          if actual != expected:
            raise AssertionError(
              f"{path} ({fmt})\nexpected: {expected!r}\nactual:   {actual!r}"
            )

        for check in checks:
          machine.succeed(f"test -f {shlex.quote(check["path"])}")
          check_stat(check["path"], "%a", check["mode"])
          check_stat(check["path"], "%u", check["owner"])
          check_stat(check["path"], "%g", check["group"])
      '';
  }
)
