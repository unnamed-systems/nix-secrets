{
  pkgs,
  nixosModule,
  shared,
  ...
}:
pkgs.testers.runNixOSTest {
  name = "generator";

  nodes.machine = {
    imports = [
      shared.minimalNoActivate
      nixosModule
    ];

    security.nix-secrets.secrets = {
      age-1.generator = "age";
      age-2.generator.age = {
        pq = true;
      };

      ssh-1.generator = "ssh";
      ssh-2.generator.ssh = {
        type = "ed25519";
        comment = "user@host";
      };
      ssh-3.generator.ssh = {
        type = "rsa";
        bits = 4096;
        comment = "user@host";
      };
      ssh-4.generator.ssh = {
        type = "rsa";
        bits = 2048;
        format = "PEM";
      };

      ssh-ed25519-1.generator = "ssh-ed25519";
      ssh-ed25519-2.generator.ssh-ed25519 = {
        comment = "user@host";
      };

      ssh-rsa-1.generator = "ssh-rsa";
      ssh-rsa-2.generator.ssh-rsa = {
        bits = 2048;
        comment = "user@host";
      };

      ssh-dkim-1.generator = "dkim";
      ssh-dkim-2.generator.dkim = {
        bits = 4096;
      };

      uuid-1.generator = "uuid";
      uuid-2.generator.uuid = {
        raw = true;
      };
      uuid-3.generator.uuid = {
        count = 5;
      };
      uuid-4.generator.uuid = {
        count = 5;
        raw = true;
      };
    };
  };

  testScript =
    { nodes, ... }:
    let
      names = builtins.attrNames nodes.machine.security.nix-secrets.secrets;
    in
    ''
      import shlex

      storage = machine.succeed("mktemp -d").strip()

      machine.succeed(f"nix-secrets -f ${shared}#shared regenerate --storage {storage} --all")

      outputs = {
        ${builtins.concatStringsSep "  \n" (
          map (
            name:
            ''"${name}": machine.succeed(f"nix-secrets -f ${shared}#shared decrypt ${name} --storage {storage} --output /dev/stdout"),''
          ) names
        )}
      }

      if len(outputs) != len(set(outputs)):
        duplicates = [
          name
          for name in outputs
          if list(outputs.values()).count(outputs[name]) > 1
        ]
        raise AssertionError(f"duplicate generated secrets: {duplicates}")


      age = {
        "age-1": False,
        "age-2": True,
      }

      for name, pq in age.items():
        lines = outputs[name].strip().splitlines()
        identity = next(line for line in reversed(lines) if not line.startswith("#"))

        prefix = "AGE-SECRET-KEY-PQ-1" if pq else "AGE-SECRET-KEY-1"

        if not identity.startswith(prefix):
          raise AssertionError(
            f"{name}: expected {prefix}, got {identity[:32]!r}"
          )


      ssh = {
        "ssh-1": ("OPENSSH", "ssh-ed25519", 256),
        "ssh-2": ("OPENSSH", "ssh-ed25519", 256),
        "ssh-3": ("OPENSSH", "ssh-rsa", 4096),
        "ssh-4": ("PEM", "ssh-rsa", 2048),
        "ssh-ed25519-1": ("OPENSSH", "ssh-ed25519", 256),
        "ssh-ed25519-2": ("OPENSSH", "ssh-ed25519", 256),
        "ssh-rsa-1": ("OPENSSH", "ssh-rsa", None),
        "ssh-rsa-2": ("OPENSSH", "ssh-rsa", 2048),

        "ssh-dkim-1": ("PKCS8", "ssh-rsa", 2048),
        "ssh-dkim-2": ("PKCS8", "ssh-rsa", 4096),
      }

      for name, (expected_format, expected_type, expected_bits) in ssh.items():
        key = outputs[name]
        prefix = {
          "OPENSSH": "-----BEGIN OPENSSH PRIVATE KEY-----",
          "PEM": "-----BEGIN RSA PRIVATE KEY-----",
          "PKCS8": "-----BEGIN PRIVATE KEY-----",
        }[expected_format]

        if not key.startswith(prefix):
          raise AssertionError(
            f"{name}: expected {expected_format} private key, got "
            f"{key.splitlines()[0]!r}"
          )

        public = machine.succeed(
          "printf '%s' " + shlex.quote(key) + " | "
          "${pkgs.openssh}/bin/ssh-keygen -y -f /dev/stdin"
        ).strip()

        key_type = public.split()[0]
        if key_type != expected_type:
          raise AssertionError(
            f"{name}: expected {expected_type}, got {key_type}"
          )

        if expected_bits is not None:
          bits = int(machine.succeed(
            "printf '%s' " + shlex.quote(public) + " | "
            "${pkgs.openssh}/bin/ssh-keygen -lf /dev/stdin"
          ).split()[0])

          if bits != expected_bits:
            raise AssertionError(
              f"{name}: expected {expected_bits} bits, got {bits}"
            )


      uuid = {
        "uuid-1": (1, 36),
        "uuid-2": (1, 32),
        "uuid-3": (5, 36),
        "uuid-4": (5, 32),
      }

      for name, (count, length) in uuid.items():
        values = outputs[name].splitlines()

        if len(values) != count:
          raise AssertionError(
            f"{name}: expected {count} values, got {len(values)}"
          )

        if any(len(value) != length for value in values):
          raise AssertionError(
            f"{name}: invalid UUID length"
          )
    '';
}
