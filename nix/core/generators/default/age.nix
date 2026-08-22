{
  lib,
  pkgs,
  ...
}:
{
  /**
    Generate an age private key.

    *Inputs:*
    - `pq` - Whether to generate a post-quantum key (`false` by default).

    *Type:*
    `age :: { pq :: Bool; } -> Derivation`

    *Examples:*
    ```nix
    generator = "age";
    ```
    ```nix
    generator.age = {
      pq = true;
    };
    ```
  */
  config.security.nix-secrets.generators.age =
    {
      pq ? false, # TODO: true, rage 0.13.0
    }:
    pkgs.writeShellScript "age-key-generator" ''
      ${pkgs.age}/bin/age-keygen${lib.optionalString pq " -pq"}
    '';
}
