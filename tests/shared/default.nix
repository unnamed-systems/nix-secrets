{ config, lib, ... }:
{
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

    nixEvalCommand = "cat ${builtins.toFile "manifest.json" config.security.nix-secrets.manifest} #";
    ciMode = {
      enableDangerously = lib.mkForce true;

      storePathIdentities = true;
      debugPackage = true;
    };

    defaultRecipients = [ "first" ];

    secrets = {
      password.recipients = lib.mkForce [ "ssh" ];
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
