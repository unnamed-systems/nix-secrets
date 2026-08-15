{
  lib,
  pkgs,
  ...
}:
{
  config.security.nix-secrets.generators.uuid =
    {
      count ? 1,
      raw ? false,
    }:
    pkgs.writeShellScript "uuid-generator" ''
      uuidgen ${
        lib.cli.toCommandLineShellGNU { } {
          random = true;
          inherit count;
        }
      }${lib.optionalString raw " | tr -d '-'"}
    '';
}
