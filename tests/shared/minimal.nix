{
  pkgs,
  config,
  lib,
  ...
}:
{
  security.nix-secrets = {
    enable = true;
    storage = ./storage;

    identityPaths = [
      ./identities/first.txt
      ./identities/id_ed25519
    ];
    recipientAliases = {
      first = builtins.readFile ./recipients/first.txt;
      ssh = builtins.readFile ./recipients/id_ed25519.pub;
    };

    nixEvalCommand = "cat ${pkgs.writeText "nix-secrets-manifest.json" config.security.nix-secrets.manifest}";
    defaultRecipients = [ "first" ];

    ciMode = {
      enableDangerously = lib.mkForce true;

      storePathIdentities = true;
      debugPackage = true;
    };
  };
}
