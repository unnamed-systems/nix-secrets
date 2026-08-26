{ lib, ... }: {
  /**
    Generate a random password from configurable character groups.

    Short-hand for [`generator.random`](#random) with a charset composed from the
    selected character groups.

    *Inputs:*
    - `length` - Length of the generated string (`32` by default).
    - `count` - Number of strings to generate (`1` by default).
    - `lowercase` - Include lowercase Latin letters (`true` by default).
    - `uppercase` - Include uppercase Latin letters (`true` by default).
    - `numbers` - Include digits (`true` by default).
    - `symbols` - Include symbols: `!@#$%^&*()-_+=` (`false` by default).
    - `extraCharset` - Additional characters to include (`""` by default).
    - `deduplicateCharset` - Remove duplicate characters from the final charset (`true` by default). Duplicates are only possible when explicitly specified in `extraCharset`.

    *Type:*
    `password :: { length :: Int; count :: Int; lowercase :: Bool; uppercase :: Bool; numbers :: Bool; symbols :: Bool; extraCharset :: String; deduplicateCharset :: Bool; } -> AttrSet`

    *Examples:*
    ```nix
    generator = "password";
    ```
    ```nix
    generator.password = {
      length = 64;
      count = 2;
      symbols = true;
    };
    ```
    ```nix
    generator.password = {
      length = 16;
      uppercase = false;
      symbols = false;
    };
    ```
    ```nix
    generator.password = {
      extraCharset = "~`";
    };
    ```
    ```nix
    generator.password = {
      lowercase = false;
      extraCharset = "aaa";
      deduplicateCharset = false;
    };
    ```
  */
  config.security.nix-secrets.generators.password =
    {
      length ? 32,
      count ? 1,
      lowercase ? true,
      uppercase ? true,
      numbers ? true,
      symbols ? false,
      extraCharset ? "",
      deduplicateCharset ? true,
    }:
    let
      raw =
        lib.optionalString lowercase "abcdefghijklmnopqrstuvwxyz"
        + lib.optionalString uppercase "ABCDEFGHIJKLMNOPQRSTUVWXYZ"
        + lib.optionalString numbers "0123456789"
        + lib.optionalString symbols "!@#$%^&*()-_+="
        + extraCharset;

      charsets =
        if deduplicateCharset then
          lib.concatStrings (lib.uniqueStrings (lib.stringToCharacters raw))
        else
          raw;
    in
    assert lib.assertMsg (charsets != "")
      "password generator: at least one character group must be enabled (lowercase, uppercase, numbers, symbols, or extraCharset)";
    {
      random = {
        inherit length count;
        charset = charsets;
      };
    };
}
