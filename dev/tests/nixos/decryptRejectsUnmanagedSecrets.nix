/*
  Secrets that exist in storage but are not present in the manifest
  must not be allowed by the decrypt command.
*/
{
  pkgs,
  nixosModule,
  shared,
  ...
}:
pkgs.testers.runNixOSTest {
  name = "decryptRejectsUnmanagedSecrets";

  nodes.machine = {
    imports = [
      nixosModule
      shared.minimalNoActivate
    ];
  };

  testScript =
    { nodes, ... }:
    let
      storage = nodes.machine.security.nix-secrets.storage;
    in
    ''
      machine.fail("nix-secrets -f ${shared}#shared decrypt password --storage ${storage} --output /dev/stdout")

      machine.fail("nix-secrets -f ${shared}#shared decrypt password --storage ${storage} --output /tmp/decrypt-test-out")
    '';
}
