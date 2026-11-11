{ moduleSystem }:
{
  lib,
  config,
  pkgs,
  ...
}:
let
  cfg = config.security.nix-secrets;

  keepEnabled = lib.filterAttrs (_name: value: value.enable);
  enabledSecrets = keepEnabled cfg.secrets;
  enabledTemplates = keepEnabled cfg.templates;
in
{
  options.security.nix-secrets.manifest = lib.mkOption {
    description = ''
      JSON manifest generated from the nix-secrets configuration.

      The manifest is used by the nix-secrets CLI to access information about
      configured secrets and templates.
    '';
    type = lib.types.functionTo lib.types.package;
    default =
      {
        withGenerators ? [ ],
        withAllGenerators ? false,
      }:
      assert withGenerators != [ ] -> !withAllGenerators;
      pkgs.writeText "nix-secrets_manifest.json" (
        builtins.toJSON {
          storage = /. + builtins.unsafeDiscardStringContext cfg.storage;
          inherit (cfg)
            identityPaths
            storagePath
            generationsDir
            generationsForUsersDir
            ;

          inherit moduleSystem;

          usePlaceholders = cfg.ciMode.enableDangerously && cfg.ciMode.usePlaceholders;

          secrets = lib.mapAttrsToList (_name: value: {
            inherit (value)
              name
              recipients
              placeholder
              neededForUsers
              path
              owner
              group
              mode
              templateKey
              isSymlink
              ;

            generator =
              if builtins.elem value.name withGenerators || withAllGenerators then
                value.generator
              else
                lib.mapNullable builtins.unsafeDiscardStringContext value.generator;
          }) enabledSecrets;

          templates = lib.mapAttrsToList (_name: value: {
            inherit (value)
              name
              content
              neededForUsers
              path
              owner
              group
              mode
              isSymlink
              ;
          }) enabledTemplates;
        }
      );
    readOnly = true;
    internal = true;
  };
}
