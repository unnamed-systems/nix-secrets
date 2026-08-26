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

      base64-1.generator = "base64";
      base64-2.generator.base64 = {
        length = 512;
        count = 3;
      };

      base64-wireguard-1.generator = "wireguard";
      base64-wireguard-2.generator.wireguard = {
        count = 3;
      };

      random-1.generator.random = {
        length = 32;
        count = 3;
      };
      random-2.generator.random = {
        length = 512;
        count = 4096;
        charset = "abc";
      };
      random-3.generator.random = {
        length = 4096;
        count = 512;
        charset = "0000000001";
      };

      random-alphanumeric-1.generator = "alphanumeric";
      random-alphanumeric-2.generator.alphanumeric = {
        length = 16;
        count = 5;
      };

      random-alnum-1.generator = "alnum";
      random-alnum-2.generator.alnum = {
        length = 16;
        count = 5;
      };

      random-hex-1.generator = "hex";
      random-hex-2.generator.hex = {
        length = 64;
        count = 5;
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
      import math
      from base64 import b64decode
      import string

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


      base64_valid = set(string.ascii_letters + string.digits + '+/=')

      base64 = {
        "base64-1": (1, 32),
        "base64-2": (3, 512),
        "base64-wireguard-1": (1, 32),
        "base64-wireguard-2": (3, 32),
      }

      for name, (count, byte_length) in base64.items():
        values = outputs[name].splitlines()
        expected_str_length = 4 * ((byte_length + 2) // 3)

        if len(values) != count:
          raise AssertionError(
            f"{name}: expected {count} values, got {len(values)}"
          )

        for value in values:
          if len(value) != expected_str_length:
            raise AssertionError(
              f"{name}: expected base64 length {expected_str_length} (from {byte_length} bytes), got {len(value)}"
            )

          if not set(value) <= base64_valid:
            raise AssertionError(
              f"{name}: value contains non-base64 characters"
            )

          decoded = b64decode(value)
          if len(decoded) != byte_length:
            raise AssertionError(
              f"{name}: decoded length {len(decoded)} != expected {byte_length}"
            )

          if "wireguard" in name:
            machine.succeed(
              f"printf '%s' {shlex.quote(value)} | ${pkgs.wireguard-tools}/bin/wg pubkey"
            )


      random = {
        "random-1": (3, 32, "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789!@#$%^&*"),
        "random-2": (4096, 512, "abc"),
        "random-3": (512, 4096, "0000000001"),
        "random-alphanumeric-1": (1, 32, "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789"),
        "random-alphanumeric-2": (5, 16, "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789"),
        "random-alnum-1": (1, 32, "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789"),
        "random-alnum-2": (5, 16, "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789"),
        "random-hex-1": (1, 32, "0123456789abcdef"),
        "random-hex-2": (5, 64, "0123456789abcdef"),
      }

      for name, (count, length, charset) in random.items():
        values = outputs[name].splitlines()

        if len(values) != count:
          raise AssertionError(
            f"{name}: expected {count} values, got {len(values)}"
          )

        for value in values:
          if len(value) != length:
            raise AssertionError(
              f"{name}: expected length {length}, got {len(value)}"
            )

          if not set(value) <= set(charset):
            raise AssertionError(
              f"{name}: value contains characters outside charset"
            )

      for name in ("random-2", "random-3"):
        _, _, charset = random[name]
        values = outputs[name].replace("\n", "")
        total = len(values)

        for char in set(charset):
          expected = charset.count(char) / len(charset)
          actual = values.count(char) / total

          deviation = 5 * math.sqrt(expected * (1 - expected) / total)

          if abs(actual - expected) > deviation:
            raise AssertionError(
              f"{name}: unexpected distribution for {char!r}: "
              f"expected {expected:.2%} ± {deviation:.2%}, "
              f"got {actual:.2%}"
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
