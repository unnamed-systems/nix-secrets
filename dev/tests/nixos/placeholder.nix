{
  pkgs,
  nixosModule,
  shared,
  ...
}:
pkgs.testers.runNixOSTest {
  name = "placeholder";

  nodes.machine = { ... }: {
    imports = [
      shared.minimal
      nixosModule
    ];

    security.nix-secrets = {
      ciMode = {
        usePlaceholders = true;
      };

      secrets = {
        file = {
          placeholder = builtins.toFile "file-placeholder" "file-success";
        };
        derivation = {
          placeholder = pkgs.runCommand "derivation-placeholder" { } "echo -n derivation-success > $out";
        };
      };
    };
  };

  testScript =
    { nodes, ... }:
    ''
      import shlex

      actual = machine.succeed(f"cat {shlex.quote(${builtins.toJSON nodes.machine.security.nix-secrets.secrets.file.path})}")
      assert actual == "file-success", f"unexpected placeholder content: {actual!r}, expected: 'file-success'"

      actual = machine.succeed(f"cat {shlex.quote(${builtins.toJSON nodes.machine.security.nix-secrets.secrets.derivation.path})}")
      assert actual == "derivation-success", f"unexpected placeholder content: {actual!r}, expected: 'derivation-success'"
    '';
}
