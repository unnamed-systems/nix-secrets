{
  pkgs,
  nixosModule,
  shared,
  ...
}:
pkgs.testers.runNixOSTest {
  name = "installPackageFalse";

  nodes.machine = {
    imports = [
      nixosModule
      shared.minimal
    ];

    security.nix-secrets.installPackage = false;
  };

  testScript = { nodes, ... }: ''
    machine.fail("which ${nodes.machine.security.nix-secrets.package.meta.mainProgram}")
  '';
}
