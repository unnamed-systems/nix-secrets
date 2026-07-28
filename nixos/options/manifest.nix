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
    description = "TODO";
    type = lib.types.str;
    default = builtins.toJSON {
      inherit (cfg) storage identityPaths;

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
