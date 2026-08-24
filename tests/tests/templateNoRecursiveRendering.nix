/*
  Secret values must not be recursively rendered.

  If a secret contains another secret's `templateKey`, it must remain
  unchanged in the rendered template. Otherwise, user-controlled secret
  values could be used to include arbitrary other secrets.
*/
{
  pkgs,
  nixosModule,
  shared,
  ...
}:
pkgs.testers.runNixOSTest (_: {
  name = "templateNoRecursiveRendering";

  nodes.machine = { config, ... }: {
    imports = [
      shared.minimal
      nixosModule
    ];

    security.nix-secrets = {
      ciMode = {
        enableDangerously = true;
        usePlaceholders = true;
      };

      secrets = {
        _1unusedSecret = {
          placeholder = "successful";
        };
        _2secret = {
          placeholder = with config.security.nix-secrets.secrets; "${_1unusedSecret}${_3unusedSecret}";
        };
        _3unusedSecret = {
          placeholder = "successful";
        };
      };

      templates.onlySecret.content = config.security.nix-secrets.secrets._2secret.templateKey;
    };
  };

  testScript =
    { nodes, ... }:
    ''
      import shlex

      template_path = ${builtins.toJSON nodes.machine.security.nix-secrets.templates.onlySecret.path}
      expected = ${
        with nodes.machine.security.nix-secrets.secrets;
        builtins.toJSON "${_1unusedSecret}${_3unusedSecret}"
      }

      machine.succeed(f"test -f {shlex.quote(template_path)}")

      actual = machine.succeed(f"cat {shlex.quote(template_path)}")
      assert actual == expected, f"unexpected template content: {actual!r}, expected: {expected!r}"
    '';
})
