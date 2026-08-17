{
  lib,
  pkgs,
  ...
}:
{
  /**
    Generate an SSH private key.

    # Inputs

    `config` (Attribute set)
    : `type`
      : SSH key type (`"ed25519"` by default).
    : `bits`
      : Number of bits for the key (`null` by default).
    : `comment`
      : Comment to add to the public key (`null` by default).
    : `format`
      : Private key format (`null` by default).

    # Type

    ```
    ssh :: { type :: String; bits :: (Int | Null); comment :: (String | Null); format :: (String | Null); } -> Derivation
    ```

    # Examples
    :::{.example}
    ## `generator.ssh` usage example

    ```nix
    generator = "ssh";

    generator.ssh = {
      type = "rsa";
      bits = 4096;
      comment = "user@host";
      format = "PEM";
    };
    ```
    :::
  */
  config.security.nix-secrets.generators.ssh =
    {
      type ? "ed25519",
      bits ? null,
      comment ? null,
      format ? null,
    }:
    pkgs.writeShellScript "ssh-key-generator" ''
      ( exec 3>&1; ${pkgs.openssh}/bin/ssh-keygen ${
        builtins.concatStringsSep " " (
          lib.cli.toCommandLine
            (optionName: {
              option = "-${optionName}";
              sep = " ";
              explicitBool = false;
              formatArg = builtins.toJSON;
            })
            {
              t = type;
              b = bits;
              C = comment;
              m = format;
              q = true;
              N = "";
              f = "/proc/self/fd/3";
            }
        )
      } <<<y >/dev/null 2>&1; true )
    '';

  /**
    Generate an Ed25519 SSH private key.

    Short-hand for `generator.ssh` with `type` set to `"ed25519"`.

    # Inputs

    `config` (Attribute set)
    : `comment`
      : Comment to add to the public key (`null` by default).
    : `format`
      : Private key format (`null` by default).

    # Type

    ```
    ssh-ed25519 :: { comment :: (String | Null); format :: (String | Null); } -> Derivation
    ```

    # Examples
    :::{.example}
    ## `generator.ssh-ed25519` usage example

    ```nix
    generator = "ssh-ed25519";

    generator.ssh-ed25519 = {
      comment = "user@host";
      format = "PEM";
    };
    ```
    :::
  */
  config.security.nix-secrets.generators.ssh-ed25519 =
    {
      comment ? null,
      format ? null,
    }:
    {
      ssh = {
        type = "ed25519";
        inherit comment format;
      };
    };

  /**
    Generate an RSA SSH private key.

    Short-hand for `generator.ssh` with `type` set to `"rsa"`.

    # Inputs

    `config` (Attribute set)
    : `bits`
      : Number of bits for the key (`null` by default).
    : `comment`
      : Comment to add to the public key (`null` by default).
    : `format`
      : Private key format (`null` by default).

    # Type

    ```
    ssh-rsa :: { bits :: (Int | Null); comment :: (String | Null); format :: (String | Null); } -> Derivation
    ```

    # Examples
    :::{.example}
    ## `generator.ssh-rsa` usage example

    ```nix
    generator = "ssh-rsa";

    generator.ssh-rsa = {
      bits = 4096;
      comment = "user@host";
      format = "PEM";
    };
    ```
    :::
  */
  config.security.nix-secrets.generators.ssh-rsa =
    {
      bits ? null,
      comment ? null,
      format ? null,
    }:
    {
      ssh = {
        type = "rsa";
        inherit bits comment format;
      };
    };
}
