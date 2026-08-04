{
  config,
  lib,
  pkgs,
  ...
}: {
  imports = [
    ./minimal.nix
  ];

  security.nix-secrets = {
    identityPaths = [
      ./identities/first.txt
      ./identities/id_ed25519
    ];
    recipientAliases = {
      first = builtins.readFile ./recipients/first.txt;
      ssh = builtins.readFile ./recipients/id_ed25519.pub;
    };

    nixEvalCommand = "cat ${builtins.toFile "nix-secrets-manifest.json" config.security.nix-secrets.manifest} #";
    ciMode = {
      enableDangerously = lib.mkForce true;

      storePathIdentities = true;
      debugPackage = true;
    };

    defaultRecipients = [ "first" ];

    secrets = {
      password = {
        recipients = lib.mkForce [ "ssh" ];
        generator = pkgs.writeShellScript "password-generator" ''
          tr -dc 'A-Za-z0-9' < /dev/urandom | head -c 16; echo "-pa$$w0rd"
        '';
      };
      passwordForUsers = {
        neededForUsers = true;
        generator = pkgs.writeShellScript "passwordForUsers-generator" ''
          tr -dc 'A-Za-z0-9' < /dev/urandom | head -c 16; echo "-pa$$w0rdF0rU$3r$"
        '';
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
