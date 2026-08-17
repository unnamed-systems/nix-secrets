{
  lib,
  pkgs,
  ...
}:
{
  /**
    Generate random UUIDs.

    # Inputs

    `config` (Attribute set)
    : `count`
      : Number of UUIDs to generate (`1` by default).
    : `raw`
      : Whether to remove hyphens from generated UUIDs (`false` by default).

    # Type

    ```
    uuid :: { count :: Int; raw :: Bool; } -> Derivation
    ```

    # Examples
    :::{.example}
    ## `generator.uuid` usage example

    ```nix
    generator = "uuid";

    generator.uuid = {
      count = 5;
      raw = true;
    };
    ```
    :::
  */
  config.security.nix-secrets.generators.uuid =
    {
      count ? 1,
      raw ? false,
    }:
    pkgs.writeShellScript "uuid-generator" ''
      uuidgen ${
        lib.cli.toCommandLineShellGNU { } {
          random = true;
          inherit count;
        }
      }${lib.optionalString raw " | tr -d '-'"}
    '';
}
