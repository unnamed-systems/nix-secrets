{
  /**
    Generate a WireGuard private key.

    Short-hand for [`generator.base64`](#base64) with `length` set to `32`.

    *Inputs:*
    - `count` - Number of keys to generate (`1` by default).

    *Type:*
    `wireguard :: { count :: Int; } -> AttrSet`

    *Examples:*
    ```nix
    generator = "wireguard";
    ```
    ```nix
    generator.wireguard = {
      count = 2;
    };
    ```
  */
  config.security.nix-secrets.generators.wireguard =
    {
      count ? 1,
    }:
    {
      base64 = {
        length = 32;
        inherit count;
      };
    };
}
