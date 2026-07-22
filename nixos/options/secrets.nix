{ lib, config, ... }:
let
  cfg = config.security.nix-secrets;

  secretSubmodule = lib.types.submodule (
    { name, config, ... }:
    {
      options = {

        name = lib.mkOption {
          description = "TODO";
          type = lib.types.str;
          default = name;
          # example = TODO;
        };

        recipients = lib.mkOption {
          description = "TODO";
          type = lib.types.listOf lib.types.str;
          default = [ ];
          apply =
            values:
            lib.uniqueStrings (builtins.concatMap (value: cfg.recipientAliases.${value} or [ value ]) values);
          # example = TODO;
        };

        # Mounting
        path = lib.mkOption {
          description = "TODO";
          type = lib.types.nullOr lib.types.str;
          default = null;
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
  options.security.nix-secrets.secrets = lib.mkOption {
    description = "TODO";
    type = lib.types.attrsOf secretSubmodule;
    default = { };
    # example = TODO;
  };
}
