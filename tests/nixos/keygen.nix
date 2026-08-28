{
  pkgs,
  nixosModule,
  shared,
  ...
}:
pkgs.testers.runNixOSTest (_: {
  name = "keygen";

  nodes.machine = {
    imports = [
      nixosModule
      shared.minimalNoActivate
    ];
  };

  testScript = ''
    def assert_valid_keypair(content):
      lines = content.splitlines()
      assert any(l.startswith("# created:") for l in lines), "missing created timestamp"
      assert sum(l.startswith("# public key: age1") for l in lines) == 1, "expected exactly 1 public key"
      assert sum(l.startswith("AGE-SECRET-KEY-1") for l in lines) == 1, "expected exactly 1 secret key"

    def secret_key(content):
      return next(l for l in content.splitlines() if l.startswith("AGE-SECRET-KEY-1"))

    output = machine.succeed("mktemp -d").strip()


    assert_valid_keypair(machine.succeed("nix-secrets keygen").strip())

    machine.succeed(f"nix-secrets keygen --output {output}/key.txt")
    assert_valid_keypair(machine.succeed(f"cat {output}/key.txt").strip())

    machine.fail(f"nix-secrets keygen --output {output}/key.txt")


    recipient_stdout = machine.succeed(f"nix-secrets keygen -y {output}/key.txt").strip()
    assert recipient_stdout.startswith("age1"), f"invalid recipient: {recipient_stdout[:20]}"

    machine.succeed(f"nix-secrets keygen -y {output}/key.txt --output {output}/key_public.txt")
    recipient_file = machine.succeed(f"cat {output}/key_public.txt").strip()
    assert recipient_file.startswith("age1"), f"invalid recipient: {recipient_file[:20]}"

    machine.fail(f"nix-secrets keygen -y {output}/nonexistent.txt")

    machine.fail(f"nix-secrets keygen -y {output}/key.txt --output {output}/key_public.txt")


    machine.succeed(f"nix-secrets keygen --output {output}/a.txt")
    machine.succeed(f"nix-secrets keygen --output {output}/b.txt")
    a = machine.succeed(f"cat {output}/a.txt").strip()
    b = machine.succeed(f"cat {output}/b.txt").strip()
    assert secret_key(a) != secret_key(b), "two keygen calls produced identical keys"
  '';
})
