{
  lib,
  pkgs,
  ...
}:
{
  /**
    Generate a random base64 string.

    *Inputs:*
    - `length` - Number of random bytes to generate (`32` by default). The output string will be `4 * ceil(length / 3)` characters long.
    - `count` - Number of strings to generate (`1` by default).

    *Type:*
    `base64 :: { length :: Int; count :: Int; } -> Derivation`

    *Examples:*
    ```nix
    generator = "base64";
    ```
    ```nix
    generator.base64 = {
      length = 64;
      count = 2;
    };
    ```
  */
  config.security.nix-secrets.generators.base64 =
    {
      length ? 32,
      count ? 1,
    }:
    pkgs.writeShellScript "base64-generator" ''
      for i in {1..${toString count}}; do
        ${lib.getExe pkgs.openssl} rand -base64 ${toString length} | tr -d '\n'
        echo
      done
    '';
}
