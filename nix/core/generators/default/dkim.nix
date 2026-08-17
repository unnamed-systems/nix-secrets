{
  /**
    Generate a DKIM private key.

    Short-hand for `generator.ssh-rsa` with `format` set to `"PKCS8"`.

    # Inputs

    `config` (Attribute set)
    : `bits`
      : Number of bits for the RSA key (`2048` by default).

    # Type

    ```
    dkim :: { bits :: (Int | Null); } -> Derivation
    ```

    # Examples
    :::{.example}
    ## `generator.dkim` usage example

    ```nix
    generator = "dkim";

    generator.dkim.bits = 4096;
    ```
    :::
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
