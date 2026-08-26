{
  /**
    Generate a random numeric string.

    Short-hand for [`generator.random`](#random) with `charset` set to `"0123456789"`.

    *Inputs:*
    - `length` - Length of the generated string (`32` by default).
    - `count` - Number of strings to generate (`1` by default).

    *Type:*
    `numeric :: { length :: Int; count :: Int; } -> AttrSet`

    *Examples:*
    ```nix
    generator = "numeric";
    ```
    ```nix
    generator.numeric = {
      length = 6;
      count = 1;
    };
    ```
  */
  config.security.nix-secrets.generators.numeric =
    {
      length ? 32,
      count ? 1,
    }:
    {
      random = {
        inherit length count;
        charset = "0123456789";
      };
    };
}
