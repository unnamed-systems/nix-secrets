{
  pkgs,
  nixosModule,
  shared,
  ...
}:
pkgs.testers.runNixOSTest {
  name = "installPackageTrue";

  nodes.machine = {
    imports = [
      nixosModule
      shared.minimal
    ];

    security.nix-secrets.installPackage = true;
  };

  testScript = { nodes, ... }: ''
    machine.succeed("which ${nodes.machine.security.nix-secrets.package.meta.mainProgram}")
  '';
}
