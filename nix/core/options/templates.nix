{ lib, ... }:
let
  templateSubmodule = lib.types.submodule (
    { name, ... }:
    {
      options = {
        enable = lib.mkOption {
          description = ''
            Whether to enable this template.

            When this option is set to `false`, the template is not included
            in the resulting manifest, like it does not exist in the first place.
          '';
          type = lib.types.bool;
          default = true;
          example = false;
        };

        name = lib.mkOption {
          description = ''
            Template name used as part of the default path.
          '';
          type = lib.types.addCheck (lib.types.pathWith {
            absolute = false;
            inStore = false;
          }) (value: builtins.isString value && "/" + value == toString (/. + value));
          default = name;
          example = "forgejo/env";
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
          example = ''
            NAME="Git Server"
            TURNSTILE_SECRET="''${config.security.nix-secrets.secrets."forgejo/turnstile/secret"}"
            TURNSTILE_SITEKEY="''${config.security.nix-secrets.secrets."forgejo/turnstile/sitekey"}"
          '';
        };

      };
    }
  );
in
{
  options.security.nix-secrets.templates = lib.mkOption {
    description = ''
      Templates managed by nix-secrets.
    '';
    type = lib.types.attrsOf templateSubmodule;
    default = { };
    # example = TODO;
  };
}
