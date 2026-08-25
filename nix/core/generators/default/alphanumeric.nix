{
  /**
    Generate a random alphanumeric string.

    Short-hand for [`generator.random`](#random) with `charset` set to `"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789"`.

    *Inputs:*
    - `length` - Length of the generated string (`32` by default).
    - `count` - Number of strings to generate (`1` by default).

    *Type:*
    `alphanumeric :: { length :: Int; count :: Int; } -> AttrSet`

    *Examples:*
    ```nix
    generator = "alphanumeric";
    ```
    ```nix
    generator.alphanumeric = {
      length = 64;
      count = 2;
    };
    ```
  */
  config.security.nix-secrets.generators.alphanumeric =
    {
      length ? 32,
      count ? 1,
    }:
    {
      random = {
        inherit length count;
        charset = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789";
      };
    };

  /**
    Generate a random alphanumeric string.

    Alias for [`generator.alphanumeric`](#alphanumeric).
  */
  config.security.nix-secrets.generators.alnum = "alphanumeric";
}
