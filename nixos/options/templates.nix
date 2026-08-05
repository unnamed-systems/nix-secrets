{ lib, config, ... }:
let
  cfg = config.security.nix-secrets;

  templateSubmodule = lib.types.submodule (
    { name, config, ... }:
    {
      options = {

        name = lib.mkOption {
          description = ''
            Template name used as part of the default path.
          '';
          type = lib.types.str;
          default = name;
          # example = TODO;
        };

        content = lib.mkOption {
          description = ''
            Template content. May be either a string or a path to file.

            Secret references can be inserted using
            `security.nix-secrets.secrets.<name>.templateKey` or by converting a secret
            option to a string: `"''${security.nix-secrets.secrets.<name>}"`.

            References are replaced with the corresponding secret values when the template
            is activated.
          '';
          type =
            let
              inputType = lib.types.either lib.types.str outputType; # TODO: inputType = lib.types.str;
              outputType = lib.types.pathWith {
                absolute = true;
                inStore = true;
              };

              transformFunction =
                value: if outputType.check value then value else builtins.toFile "nix-secrets-template" value;
            in
            lib.types.coercedTo inputType transformFunction outputType;
          # example = TODO;
        };

        neededForUsers = lib.mkOption {
          description = ''
            Whether the template must be available before users and groups are created.

            Enable this for templates referenced before or during user creation.
          '';
          type = lib.types.bool;
          default = false;
          example = true;
        };

        # Mounting
        path = lib.mkOption {
          description = ''
            Path where the rendered template file will be mounted.

            By default, the template is mounted under "/run/nix-secrets/templates", or under
            "/run/nix-secrets-for-users/templates" when `neededForUsers` is enabled.
          '';
          type = lib.types.str;
          # Use separate directories because activation scripts with
          # `beforeUsers = true` and `beforeUsers = false` run independently.
          default = "/run/nix-secrets${lib.optionalString config.neededForUsers "-for-users"}/templates/${config.name}";
          # example = TODO;
        };

        mode = lib.mkOption {
          description = ''
            Permissions of the file created at `path`.
          '';
          type = lib.types.str;
          default = "0400";
          # example = TODO;
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
          default = 0;
          # example = TODO;
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
          default = 0;
          # example = TODO;
        };

      };
    }
  );
in
{
  options.security.nix-secrets.templates = lib.mkOption {
    description = ''
      Templates managed by Nix-Secrets.
    '';
    type = lib.types.attrsOf templateSubmodule;
    default = { };
    # example = TODO;
  };
}
