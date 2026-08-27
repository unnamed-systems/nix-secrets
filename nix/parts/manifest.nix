{ moduleSystem }:
{
  lib,
  config,
  ...
}:
let
  cfg = config.security.nix-secrets;
in
{
  options.security.nix-secrets.manifest = lib.mkOption {
    description = ''
      JSON manifest generated from the nix-secrets configuration.

      The manifest is used by the nix-secrets CLI to access information about
      configured secrets and templates.
    '';
    type = lib.types.str;
    default = builtins.toJSON {
      inherit (cfg) storage identityPaths storagePath;

      inherit moduleSystem;

      usePlaceholders = cfg.ciMode.enableDangerously && cfg.ciMode.usePlaceholders;

      secrets = lib.mapAttrsToList (_name: value: {
        inherit (value)
          name
          recipients
          placeholder
          generator
          neededForUsers
          path
          owner
          group
          mode
          templateKey
          ;
      }) cfg.secrets;

      templates = lib.mapAttrsToList (_name: value: {
        inherit (value)
          name
          content
          neededForUsers
          path
          owner
          group
          mode
          ;
      }) cfg.templates;
    };
    readOnly = true;
    internal = true;
  };
}
