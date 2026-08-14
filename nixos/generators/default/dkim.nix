{
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
