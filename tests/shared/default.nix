{ config, lib, ... }: {
  imports = [
    ./minimal.nix
  ];

  security.nix-secrets = {
    identityPaths = [ ./identities/first.txt ];
    recipientAliases = {
      first = builtins.readFile ./recipients/first.txt;
    };

    nixEvalCommand = "cat ${builtins.toFile "manifest.json" config.security.nix-secrets.manifest} #";
    ciMode = {
      enableDangerously = lib.mkForce true;

      storePathIdentities = true;
      debugPackage = true;
    };

    secrets = {
      password.recipients = [ "first" ];
      passwordForUsers = {
        recipients = [ "first" ];
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
