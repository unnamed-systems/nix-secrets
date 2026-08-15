{
  lib,
  pkgs,
  ...
}:
{
  config.security.nix-secrets.generators.age =
    {
      pq ? false, # TODO: true, rage 0.13.0
    }:
    pkgs.writeShellScript "age-key-generator" ''
      ${pkgs.age}/bin/age-keygen${lib.optionalString pq " -pq"}
    '';
}
