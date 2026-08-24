{
  /**
    Generate a DKIM private key.

    Short-hand for `generator.ssh-rsa` with `format` set to `"PKCS8"`.

    *Inputs:*
    - `bits` - Number of bits for the RSA key (`2048` by default).

    *Type:*
    `dkim :: { bits :: (Int | Null); } -> AttrSet`

    *Examples:*
    ```nix
    generator = "dkim";
    ```
    ```nix
    generator.dkim = {
      bits = 4096;
    };
    ```
  */
  config.security.nix-secrets.generators.dkim =
    {
      bits ? 2048,
    }:
    {
      ssh-rsa = {
        inherit bits;
        format = "PKCS8";
      };
    };
}
