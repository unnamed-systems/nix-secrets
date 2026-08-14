{
  lib,
  pkgs,
  ...
}:
{
  config.security.nix-secrets.generators = {
    ssh =
      {
        type ? "ed25519",
        bits ? null,
        comment ? null,
        format ? null,
      }:
      pkgs.writeShellScript "ssh-key-generator" ''
        ( exec 3>&1; ${pkgs.openssh}/bin/ssh-keygen ${
          builtins.concatStringsSep " " (
            lib.cli.toCommandLine
              (optionName: {
                option = "-${optionName}";
                sep = " ";
                explicitBool = false;
                formatArg = builtins.toJSON;
              })
              {
                t = type;
                b = bits;
                C = comment;
                m = format;
                q = true;
                N = "";
                f = "/proc/self/fd/3";
              }
          )
        } <<<y >/dev/null 2>&1; true )
      '';

    ssh-ed25519 =
      {
        bits ? null,
        comment ? null,
        format ? null,
      }:
      {
        ssh = {
          type = "ed25519";
          inherit bits comment format;
        };
      };

    ssh-rsa =
      {
        bits ? null,
        comment ? null,
        format ? null,
      }:
      {
        ssh = {
          type = "rsa";
          inherit bits comment format;
        };
      };
  };
}
