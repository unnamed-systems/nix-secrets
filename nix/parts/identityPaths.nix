{ moduleSystem }:
assert builtins.elem moduleSystem [
  "hjem"
  "home-manager"
  "nix-darwin"
  "nixos"
];
{ lib, config, ... }:
let
  cfg = config.security.nix-secrets;

  defaultPath =
    escape:
    let
      subpath = "/nix-secrets/identities.txt";
      path =
        {
          hjem =
            if escape then
              "\${config.xdg.config.directory}${subpath}"
            else
              "${config.xdg.config.directory}${subpath}";
          home-manager =
            if escape then "\${config.xdg.configHome}${subpath}" else "${config.xdg.configHome}${subpath}";
          nix-darwin = "/Library/Application Support${subpath}";
          nixos = "/var/lib${subpath}";
        }
        .${moduleSystem};
    in
    if escape then builtins.toJSON path else path;

  sshHostKeysSupported = builtins.elem moduleSystem [
    "nix-darwin"
    "nixos"
  ];
  supportedKeyTypes = [
    "ed25519"
    "rsa"
  ];
in
{
  options.security.nix-secrets = {

    identityPaths = lib.mkOption {
      description = ''
        Paths to identity files used for secret encryption and decryption.

        For security reasons, identity files are kept outside of the Nix store.

        Defaults to the identity path `${defaultPath true}`${lib.optionalString sshHostKeysSupported " and the RSA and ED25519 SSH keys from the `services.openssh.hostKeys`"}.
      '';
      type = lib.types.listOf (
        lib.types.pathWith {
          inStore = if cfg.ciMode.enableDangerously && cfg.ciMode.storePathIdentities then null else false;
          absolute = null;
        }
      );
      default = [
        (defaultPath false)
      ]
      ++ lib.optionals sshHostKeysSupported (
        map (k: k.path) (
          builtins.filter (k: builtins.elem k.type supportedKeyTypes) config.services.openssh.hostKeys
        )
      );
      defaultText = lib.literalExpression ''
        [ ${defaultPath true} ]${lib.optionalString sshHostKeysSupported ''

          ++ map (k: k.path) (
            builtins.filter
            (k: builtins.elem k.type [${
              lib.concatMapStringsSep " " lib.strings.escapeNixString supportedKeyTypes
            })
            config.services.openssh.hostKeys
          )
        ''}
      '';
      example = [
        "/var/lib/nix-secrets/identities.txt"
        "/home/user/keys.txt"
        "/home/user/.ssh/id_ed25519"
      ];
    };

  };
}
