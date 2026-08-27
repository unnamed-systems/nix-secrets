{ moduleSystem }:
{ lib, pkgs, ... }:
let
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
              default =
                if moduleSystem == "home-manager" then
                  "${
                    if pkgs.stdenv.hostPlatform.isDarwin then
                      "$(${lib.getExe pkgs.getconf} DARWIN_USER_TEMP_DIR)"
                    else
                      "\${XDG_RUNTIME_DIR}/home/user"
                  }/nix-secrets/${attr}/${config.name}"
                else
                  "/run/nix-secrets${lib.optionalString config.neededForUsers "-for-users"}/${attr}/${config.name}";
              defaultText =
                if moduleSystem == "home-manager" then
                  lib.literalExpression ''"''${if pkgs.stdenv.hostPlatform.isDarwin then "$(''${lib.getExe pkgs.getconf} DARWIN_USER_TEMP_DIR)" else "\''${XDG_RUNTIME_DIR}"}/nix-secrets/${attr}/${config.name}"''
                else
                  lib.literalExpression ''"/run/nix-secrets''${lib.optionalString config.neededForUsers "-for-users"}/${attr}/${config.name}"'';
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
    generationsDir = lib.mkOption {
      description = "TODO";
      type = lib.types.str;
      default = "${runtimeDir}/nix-secrets.d";
      # example = TODO;
    };
    generationsForUsersDir = lib.mkOption {
      description = "TODO";
      type = lib.types.str;
      default = "${runtimeDir}/nix-secrets-for-users.d";
      # example = TODO;
    };
  };
}
