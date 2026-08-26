{
  pkgs,
  nixosModule,
  shared,
  ...
}:
pkgs.testers.runNixOSTest {
  name = "edit";

  nodes.machine = {
    imports = [
      nixosModule
      shared.defaultNoActivate
    ];

    environment.variables.EDITOR = "-";
  };

  testScript = ''
    storage = machine.succeed("mktemp -d").strip()

    machine.succeed(f"cp -r ${shared.outPath}/storage/* {storage}/")

    new_value = "edited-secret-value-12345"
    machine.succeed(
      f"printf '%s' '{new_value}' | "
      f"nix-secrets -f ${shared}#shared edit password --storage {storage}"
    )

    decrypted = machine.succeed(
      f"nix-secrets -f ${shared}#shared decrypt password --storage {storage} --output /dev/stdout"
    ).strip()

    assert decrypted == new_value, f"expected {new_value!r}, got {decrypted!r}"

    machine.succeed(
      f"printf '%s' '{new_value}' | "
      f"nix-secrets -f ${shared}#shared edit password --storage {storage}"
    )

    decrypted2 = machine.succeed(
      f"nix-secrets -f ${shared}#shared decrypt password --storage {storage} --output /dev/stdout"
    ).strip()

    assert decrypted2 == new_value, f"unchanged secret was re-encrypted: {decrypted2!r}"
  '';
}
