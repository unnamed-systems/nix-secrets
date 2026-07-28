{ lib, config, ... }:
let
  cfg = config.security.nix-secrets;

  templateSubmodule = lib.types.submodule (
    { name, config, ... }:
    {
      options = {

        name = lib.mkOption {
          description = "TODO";
          type = lib.types.str;
          default = name;
          # example = TODO;
        };

        content = lib.mkOption {
          description = "TODO";
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
          description = "TODO";
          type = lib.types.bool;
          default = false;
          example = true;
        };

        # Mounting
        path = lib.mkOption {
          description = "TODO";
          type = lib.types.str;
          # Use separate directories because activation scripts with
          # `beforeUsers = true` and `beforeUsers = false` run independently.
          default = "/run/nix-secrets${lib.optionalString config.neededForUsers "-for-users"}/templates/${config.name}";
          # example = TODO;
        };

        mode = lib.mkOption {
          description = "TODO";
          type = lib.types.str;
          default = "0400";
          # example = TODO;
        };

        owner = lib.mkOption {
          description = "TODO";
          type = lib.types.oneOf [
            lib.types.int
            lib.types.str
          ];
          default = 0;
          # example = TODO;
        };

        group = lib.mkOption {
          description = "TODO";
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
    description = "TODO";
    type = lib.types.attrsOf templateSubmodule;
    default = { };
    # example = TODO;
  };
}
