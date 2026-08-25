{
  lib,
  pkgs,
  ...
}:
{
  /**
    Generate a random string.

    *Inputs:*
    - `length` - Length of the generated string (`32` by default).
    - `charset` - Characters to use for generating the string (`"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789!@#$%^&*"` by default).
    - `count` - Number of strings to generate (`1` by default).

    *Type:*
    `random :: { length :: Int; charset :: String; count :: Int; } -> Derivation`

    *Examples:*
    ```nix
    generator = "random";
    ```
    ```nix
    generator.random = {
      length = 64;
      charset = "abcdefghijklmnopqrstuvwxyz0123456789";
      count = 2;
    };
    ```
  */
  config.security.nix-secrets.generators.random =
    {
      length ? 32,
      charset ? "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789!@#$%^&*",
      count ? 1,
    }:
    pkgs.writeShellScript "random-string-generator" ''
      ${lib.getExe (pkgs.callPackage ./src/package.nix { })} \
        --length ${lib.escapeShellArg (toString length)} \
        --charset ${lib.escapeShellArg charset} \
        --count ${lib.escapeShellArg (toString count)}
    '';
}
