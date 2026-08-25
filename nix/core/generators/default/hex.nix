{
  /**
    Generate a random hexadecimal string.

    Short-hand for [`generator.random`](#random) with `charset` set to `"0123456789abcdef"`.

    *Inputs:*
    - `length` - Length of the generated string (`32` by default).
    - `count` - Number of strings to generate (`1` by default).

    *Type:*
    `hex :: { length :: Int; count :: Int; } -> AttrSet`

    *Examples:*
    ```nix
    generator = "hex";
    ```
    ```nix
    generator.hex = {
      length = 64;
      count = 2;
    };
    ```
  */
  config.security.nix-secrets.generators.hex =
    {
      length ? 32,
      count ? 1,
    }:
    {
      random = {
        inherit length count;
        charset = "0123456789abcdef";
      };
    };
}
