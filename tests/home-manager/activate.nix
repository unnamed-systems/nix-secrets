{
  pkgs,
  homeManagerModule,
  shared,
  devInputs,
  ...
}:
pkgs.testers.runNixOSTest (
  { lib, ... }:
  {
    name = "home-manager-activate";

    nodes.machine = {
      imports = [
        devInputs.home-manager.nixosModules.home-manager
      ];

      users.users.user = {
        isNormalUser = true;
        linger = true;
        home = "/home/user";
        extraGroups = [ "wheel" ];
      };

      home-manager.users.user = {
        home.stateVersion = "26.11";

        imports = [
          homeManagerModule
          shared.default
        ];
      };
    };

    testScript =
      { nodes, ... }:
      let
        checks =
          map
            (secret: {
              inherit (secret) path owner group;
              mode = lib.removePrefix "0" secret.mode;
            })
            (
              (builtins.attrValues nodes.machine.home-manager.users.user.security.nix-secrets.secrets)
              ++ (builtins.attrValues nodes.machine.home-manager.users.user.security.nix-secrets.templates)
            );
      in
      ''
        import json
        import shlex

        checks = json.loads(${builtins.toJSON (builtins.toJSON checks)})

        def stat(path, fmt):
          return machine.succeed(
            f"stat -L -c {shlex.quote(fmt)} {shlex.quote(path)}"
          ).strip()

        def check_stat(path, fmt, expected):
          actual = stat(path, fmt)
          if actual != expected:
            raise AssertionError(
              f"{path} ({fmt})\nexpected: {expected!r}\nactual:   {actual!r}"
            )

        machine.wait_for_unit("default.target")

        for check in checks:
          machine.succeed(f"test -f {shlex.quote(check["path"])}")
          check_stat(check["path"], "%a", check["mode"])
          check_stat(check["path"], "%u" if type(check["owner"]) == int else "%U", str(check["owner"]))
          check_stat(check["path"], "%g" if type(check["group"]) == int else "%G", str(check["group"]))
      '';
  }
)
