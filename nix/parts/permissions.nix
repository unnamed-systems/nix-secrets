{ moduleSystem }:
{ lib, config, ... }:
let
  cfg = config.security.nix-secrets;

  mkPermissions =
    _attr:
    lib.mkOption {
      type = lib.types.attrsOf (
        lib.types.submodule {
          options = {

            mode = lib.mkOption {
              description = ''
                Permissions of the file created at `path`.
              '';
              type = lib.types.str;
              default = cfg.defaultMode;
              defaultText = lib.literalExpression "cfg.defaultMode";
              example = "0660";
            };

            owner = lib.mkOption {
              description = ''
                Owner of the file created at `path`.

                May be specified as either a user name or a numeric UID.

                Cannot be set when `neededForUsers` is enabled because users and groups are
                not available at this stage of activation. In this case, the file owner is
                set to `root`.
              '';
              type = lib.types.oneOf [
                lib.types.int
                lib.types.str
              ];
              default = cfg.defaultOwner;
              defaultText = lib.literalExpression "cfg.defaultOwner";
              example = "forgejo";
            };

            group = lib.mkOption {
              description = ''
                Group of the file created at `path`.

                May be specified as either a group name or a numeric GID.

                Cannot be set when `neededForUsers` is enabled because users and groups are
                not available at this stage of activation. In this case, the file group is
                set to `root`.
              '';
              type = lib.types.oneOf [
                lib.types.int
                lib.types.str
              ];
              default = cfg.defaultGroup;
              defaultText = lib.literalExpression "cfg.defaultGroup";
              example = "users";
            };

          };
        }
      );
    };
in
{
  options.security.nix-secrets = lib.genAttrs [ "secrets" "templates" ] mkPermissions // {

    defaultMode = lib.mkOption {
      description = ''
        Default permissions for secrets and templates.

        Applied to both unless overridden by
        `security.nix-secrets.secrets.<name>.mode` or
        `security.nix-secrets.templates.<name>.mode`.
      '';
      type = lib.types.str;
      default = "0400";
      example = "0660";
    };

    defaultOwner = lib.mkOption {
      description = ''
        Default owner for secrets and templates.

        May be specified as either a user name or a numeric UID.

        Applied to both unless overridden by
        `security.nix-secrets.secrets.<name>.owner` or
        `security.nix-secrets.templates.<name>.owner`.
      '';
      type = lib.types.oneOf [
        lib.types.int
        lib.types.str
      ];
      default =
        if moduleSystem == "home-manager" then
          config.home.username
        else if moduleSystem == "hjem" then
          config.user
        else
          0;
      defaultText =
        if moduleSystem == "home-manager" then
          lib.literalExpression "config.home.username"
        else if moduleSystem == "hjem" then
          lib.literalExpression "config.user"
        else
          0;
      example = "forgejo";
    };

    defaultGroup = lib.mkOption {
      description = ''
        Default group for secrets and templates.

        May be specified as either a group name or a numeric GID.

        Applied to both unless overridden by
        `security.nix-secrets.secrets.<name>.group` or
        `security.nix-secrets.templates.<name>.group`.
      '';
      type = lib.types.oneOf [
        lib.types.int
        lib.types.str
      ];
      default =
        if moduleSystem == "home-manager" then
          config._module.specialArgs.osConfig.users.users.${config.home.username}.group
            or (throw "cannot determine the default group in standalone Home Manager: set `security.nix-secrets.defaultGroup` explicitly")
        else if moduleSystem == "hjem" then
          config._module.specialArgs.osConfig.users.users.${config.user}.group
            or (throw "cannot determine the default group in standalone Hjem: set `security.nix-secrets.defaultGroup` explicitly")
        else
          0;
      defaultText =
        if moduleSystem == "home-manager" then
          lib.literalExpression ''config._module.specialArgs.osConfig.users.users.''${config.home.username}.group or (throw "cannot determine the default group in standalone Home Manager: set `security.nix-secrets.defaultGroup` explicitly")''
        else if moduleSystem == "hjem" then
          lib.literalExpression ''config._module.specialArgs.osConfig.users.users.''${config.user}.group or (throw "cannot determine the default group in standalone Hjem: set `security.nix-secrets.defaultGroup` explicitly")''
        else
          0;
      example = "users";
    };

  };
}
