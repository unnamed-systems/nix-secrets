{ moduleSystem }:
{ lib, config, ... }:
let
  cfg = config.security.nix-secrets;

  descriptions = {
    secrets = ''
      Path where the decrypted secret will be mounted.

      ${
        if moduleSystem == "home-manager" then
          ''
            By default, the secret is mounted under "''${XDG_RUNTIME_DIR}/nix-secrets/secrets" if
            on non-Darwin systems, or under "$(getconf DARWIN_USER_TEMP_DIR)/nix-secrets/secrets"
            on Darwin.''
        else
          ''
            By default, the secret is mounted under "/run/nix-secrets/secrets", or under
            "/run/nix-secrets-for-users/secrets" when `neededForUsers` is enabled.''
      }

      Secrets using the default paths are activated atomically together. When a
      custom path is specified, secrets are updated individually and atomicity is
      only guaranteed for each individual secret.
    '';
    templates = ''
      Path where the rendered template will be mounted.

      ${
        if moduleSystem == "home-manager" then
          ''
            By default, the template is mounted under "''${XDG_RUNTIME_DIR}/nix-secrets/templates" if
            on non-Darwin systems, or under "$(getconf DARWIN_USER_TEMP_DIR)/nix-secrets/templates"
            on Darwin.''
        else
          ''
            By default, the template is mounted under "/run/nix-secrets/templates", or under
            "/run/nix-secrets-for-users/templates" when `neededForUsers` is enabled.''
      }

      Templates using the default paths are activated atomically together. When a
      custom path is specified, templates are updated individually and atomicity is
      only guaranteed for each individual template.
    '';
  };

  mkPath =
    attr:
    lib.mkOption {
      type = lib.types.attrsOf (
        lib.types.submodule (
          { config, ... }: {
            options.path = lib.mkOption {
              description = descriptions.${attr};
              type = lib.types.str;
              # Use separate directories because activation scripts with
              # `beforeUsers = true` and `beforeUsers = false` run independently.
              default = "${
                if config.neededForUsers then cfg.baseForUsersDir else cfg.baseDir
              }/${attr}/${config.name}";
              defaultText = lib.literalExpression ''"''${if config.neededForUsers then cfg.baseForUsersDir else cfg.baseDir}/${attr}/${config.name}"'';
              example = "${if moduleSystem == "home-manager" then "$HOME/.config" else "/var/lib"}/forgejo/${
                if attr == "secrets" then "token" else ".env"
              }";
            };
          }
        )
      );
    };

  runtimeDir = if moduleSystem == "home-manager" then "{{RUNTIME_DIR}}" else "/run";
in
{
  options.security.nix-secrets = lib.genAttrs [ "secrets" "templates" ] mkPath // {
    baseDir = lib.mkOption {
      description = ''
        Default directory where secrets and templates are mounted.

        Each secret or template without a custom `path` is placed under this directory.
      '';
      type = lib.types.str;
      default = "${
        if moduleSystem == "home-manager" then
          if lib.hasPrefix "~/" config.xdg.dataHome then
            "${config.home.homeDirectory}/${lib.removePrefix "~/" config.xdg.dataHome}"
          else
            config.xdg.dataHome
        else
          "/run"
      }/nix-secrets";
      defaultText = lib.literalExpression (
        if moduleSystem == "home-manager" then
          ''"''${if lib.hasPrefix "~/" config.xdg.dataHome then "''${config.home.homeDirectory}/''${lib.removePrefix "~/" config.xdg.dataHome}" else config.xdg.dataHome}/nix-secrets"''
        else
          ''"/run/nix-secrets"''
      );
      # example = TODO;
    };
    baseForUsersDir = lib.mkOption {
      description = ''
        Default directory where secrets and templates with `neededForUsers = true`
        are mounted.

        Serves the same purpose as `baseDir` but for the `neededForUsers`
        activation phase.
      '';
      type = lib.types.str;
      default = "${cfg.baseDir}-for-users";
      defaultText = lib.literalExpression ''"''${cfg.baseDir}-for-users"'';
      # example = TODO;
    };
    generationsDir = lib.mkOption {
      description = ''
        Directory where generations are stored for the standard activation phase.

        A generation holds the set of secrets and templates that were active at
        a given point in time, allowing rollback to a previous state.
      '';
      type = lib.types.str;
      default = "${runtimeDir}/nix-secrets.d";
      # example = TODO;
    };
    generationsForUsersDir = lib.mkOption {
      description = ''
        Directory where generations are stored for the `neededForUsers`
        activation phase.

        A generation holds the set of secrets and templates that were active at
        a given point in time, allowing rollback to a previous state.
      '';
      type = lib.types.str;
      default = "${runtimeDir}/nix-secrets-for-users.d";
      # example = TODO;
    };
  };
}
