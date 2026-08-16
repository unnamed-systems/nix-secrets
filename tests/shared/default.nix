{
  config,
  lib,
  pkgs,
  ...
}:
{
  imports = [
    ./minimal.nix
  ];

  security.nix-secrets = {
    secrets = {
      password = {
        recipients = lib.mkForce [ "ssh" ];
        generator = pkgs.writeShellScript "password-generator" ''
          tr -dc 'A-Za-z0-9' < /dev/urandom | head -c 16; echo "-pa$$w0rd"
        '';
      };
      passwordForUsers = {
        neededForUsers = true;
      };
    };

    templates = {
      text.content = ''
        password = ${config.security.nix-secrets.secrets.password}
      '';
      textForUsers = {
        content = ''
          password = ${config.security.nix-secrets.secrets.passwordForUsers}
        '';
        neededForUsers = true;
      };

      file.content = builtins.toFile "file-template" ''
        password = ${config.security.nix-secrets.secrets.password}
      '';
      fileForUsers = {
        content = builtins.toFile "file-template" ''
          password = ${config.security.nix-secrets.secrets.passwordForUsers}
        '';
        neededForUsers = true;
      };
    };
  };
}
