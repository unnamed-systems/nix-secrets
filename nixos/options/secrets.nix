{ lib, ... }:
let
  secretSubmodule = lib.types.submodule (
    { name, config, ... }:
    {
      options = {

        file = lib.mkOption {
          description = "TODO";
          type = lib.types.path;
          # default = TODO;
          # example = TODO;
        };

        # Mounting
        path = lib.mkOption {
          description = "TODO";
          type = lib.types.str;
          # default = TODO;
          # example = TODO;
        };

        mode = lib.mkOption {
          description = "TODO";
          type = lib.types.path;
          # default = TODO;
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
  options.security.nix-secrets.secrets = lib.mkOption {
    description = "TODO";
    type = lib.types.attrsOf secretSubmodule;
    default = { };
    # example = TODO;
  };
}
