{
  pkgs,
  nixosModule,
  shared,
  ...
}:
pkgs.testers.runNixOSTest {
  name = "triggers";

  nodes.machine = {
    imports = [
      nixosModule
      shared.minimalNoActivate
    ];

    security.nix-secrets = {
      ciMode.usePlaceholders = true;

      secrets.password = {
        placeholder = "password";
        isSymlink = false;
        reloadUnits = [ "trigger-reload.service" ];
        restartUnits = [ "trigger-restart.service" ];
      };

      secrets.regular = {
        placeholder = "regular";
        isSymlink = false;
        reloadUnits = [ "trigger-reload.service" ];
        restartUnits = [ "trigger-restart.service" ];
      };

      templates.text = {
        content = "template";
        reloadUnits = [ "trigger-reload.service" ];
        restartUnits = [ "trigger-restart.service" ];
      };
    };

    systemd.services.trigger-reload = {
      wantedBy = [ "multi-user.target" ];
      serviceConfig = {
        Type = "oneshot";
        RemainAfterExit = true;
        ExecStart = "touch /tmp/reload-triggered";
        ExecReload = "touch /tmp/reload-triggered";
      };
    };

    systemd.services.trigger-restart = {
      wantedBy = [ "multi-user.target" ];
      serviceConfig = {
        Type = "oneshot";
        RemainAfterExit = true;
        ExecStart = "touch /tmp/restart-triggered";
      };
    };
  };

  testScript = { nodes, ... }: ''
    import shlex

    activate = ${builtins.toJSON (nodes.machine.security.nix-secrets.activate.command false)}

    password_path = ${builtins.toJSON nodes.machine.security.nix-secrets.secrets.password.path}
    regular_path = ${builtins.toJSON nodes.machine.security.nix-secrets.secrets.regular.path}
    template_path = ${builtins.toJSON nodes.machine.security.nix-secrets.templates.text.path}

    def clear_triggers():
      machine.succeed("rm -f /tmp/reload-triggered /tmp/restart-triggered")

    def assert_triggers(expected, label):
      reload_triggered = machine.succeed("test -f /tmp/reload-triggered && echo yes || echo no").strip() == "yes"
      restart_triggered = machine.succeed("test -f /tmp/restart-triggered && echo yes || echo no").strip() == "yes"
      assert reload_triggered == expected, f"{label}: unexpected reload trigger"
      assert restart_triggered == expected, f"{label}: unexpected restart trigger"

    clear_triggers()
    machine.succeed(activate)
    machine.succeed(f"test -f {shlex.quote(password_path)}")
    machine.succeed(f"test -f {shlex.quote(regular_path)}")
    machine.succeed(f"test -f {shlex.quote(template_path)}")
    assert_triggers(True, "missing paths")

    clear_triggers()
    machine.succeed(activate)
    assert_triggers(False, "unchanged secrets")

    clear_triggers()
    machine.succeed(f"printf wrong > {shlex.quote(password_path)}")
    machine.succeed(activate)
    assert_triggers(True, "changed content")

    clear_triggers()
    machine.succeed(f"chmod 0666 {shlex.quote(password_path)}")
    machine.succeed(activate)
    assert_triggers(True, "changed permissions")

    clear_triggers()
    machine.succeed(f"chown 1000:1000 {shlex.quote(password_path)}")
    machine.succeed(activate)
    assert_triggers(True, "changed ownership")

    clear_triggers()
    machine.succeed(f"cp {shlex.quote(regular_path)} /tmp/regular-content")
    machine.succeed(f"rm -f {shlex.quote(regular_path)}")
    machine.succeed(f"ln -s /tmp/regular-content {shlex.quote(regular_path)}")
    machine.succeed(activate)
    assert_triggers(True, "changed isSymlink")
    machine.succeed(f"test -f {shlex.quote(regular_path)}")
    machine.succeed(f"! test -L {shlex.quote(regular_path)}")
  '';
}
