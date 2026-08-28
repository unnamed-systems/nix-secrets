{
  pkgs,
  nixosModule,
  shared,
  ...
}:
pkgs.testers.runNixOSTest {
  name = "decrypt";

  nodes.machine = {
    imports = [
      nixosModule
      shared.defaultNoActivate
    ];
  };

  testScript =
    { nodes, ... }:
    let
      storage = nodes.machine.security.nix-secrets.storage;
    in
    ''
      stdout = machine.succeed("nix-secrets -f ${shared}#shared decrypt password --storage ${storage} --output /dev/stdout").strip()
      assert stdout, "decrypt to stdout returned empty"

      machine.succeed("nix-secrets -f ${shared}#shared decrypt password --storage ${storage} --output /tmp/decrypt-test-out")
      file_content = machine.succeed("cat /tmp/decrypt-test-out").strip()
      assert file_content, "decrypt to file returned empty"

      assert stdout == file_content, "stdout and file output differ"

      machine.fail("nix-secrets -f ${shared}#shared decrypt nonexistent --storage ${storage} --output /dev/stdout")

      machine.fail("nix-secrets -f ${shared}#shared decrypt password --output /dev/stdout")
    '';
}
